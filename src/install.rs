// Copyright 2026 sufiyan-sabeel
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0

//! Copying bundles onto disk, checking what landed there, and migrating a
//! v0.1.x flat install into the bundle layout.

use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Component, Path, PathBuf};

use crate::bundle::BundleId;
use crate::{assets, generated};

/// How to install.
#[derive(Debug, Clone)]
pub struct InstallOptions {
    /// Directory each bundle root is written under: `<dest>/<bundle>/...`.
    pub dest: PathBuf,
    /// Bundles to install.
    pub bundles: Vec<BundleId>,
    /// Overwrite files that exist but differ from the embedded copy.
    pub force: bool,
    /// Report what would happen without touching the filesystem.
    pub dry_run: bool,
}

impl InstallOptions {
    /// Options for installing every bundle into `dest`.
    pub fn new(dest: impl Into<PathBuf>) -> Self {
        Self {
            dest: dest.into(),
            bundles: BundleId::default_set(),
            force: false,
            dry_run: false,
        }
    }

    /// Narrow the install to specific bundles.
    pub fn bundles(mut self, bundles: Vec<BundleId>) -> Self {
        self.bundles = bundles;
        self
    }

    /// Overwrite locally modified files.
    pub fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// Do everything except write.
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
}

/// What an install run did.
#[derive(Debug, Default, Clone)]
pub struct InstallReport {
    /// Bundles considered.
    pub bundles: Vec<BundleId>,
    /// Files written (or that would be written, on a dry run).
    pub written: Vec<PathBuf>,
    /// Files already identical to the expected copy.
    pub unchanged: Vec<PathBuf>,
    /// Files that differ locally and were left alone because `force` was off.
    pub skipped: Vec<PathBuf>,
    /// Whether this was a dry run.
    pub dry_run: bool,
}

impl InstallReport {
    /// Files considered in total.
    pub fn total(&self) -> usize {
        self.written.len() + self.unchanged.len() + self.skipped.len()
    }

    /// One-line summary suitable for a terminal.
    pub fn summary(&self) -> String {
        let verb = if self.dry_run { "would write" } else { "wrote" };
        format!(
            "{verb} {}, unchanged {}, skipped {} across {} bundle(s)",
            self.written.len(),
            self.unchanged.len(),
            self.skipped.len(),
            self.bundles.len()
        )
    }
}

/// Result of comparing an installed tree against the embedded bundles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyStatus {
    /// File exists and matches byte for byte.
    Ok,
    /// File does not exist.
    Missing,
    /// File exists but differs.
    Modified,
}

/// What [`verify`] found.
#[derive(Debug, Default, Clone)]
pub struct VerifyReport {
    /// Bundles considered.
    pub bundles: Vec<BundleId>,
    /// Files that match.
    pub ok: Vec<PathBuf>,
    /// Files that are absent.
    pub missing: Vec<PathBuf>,
    /// Files that differ from the expected copy.
    pub modified: Vec<PathBuf>,
}

impl VerifyReport {
    /// True when nothing is missing and nothing differs.
    pub fn is_clean(&self) -> bool {
        self.missing.is_empty() && self.modified.is_empty()
    }

    /// Files considered in total.
    pub fn total(&self) -> usize {
        self.ok.len() + self.missing.len() + self.modified.len()
    }
}

/// What [`migrate`] moved out of a v0.1.x flat install.
#[derive(Debug, Default, Clone)]
pub struct MigrateReport {
    /// Files moved, as `(from, to)`.
    pub moved: Vec<(PathBuf, PathBuf)>,
    /// Files that differ from the embedded copy and were therefore left in place.
    pub left: Vec<PathBuf>,
    /// True when nothing was found at the old layout.
    pub nothing_to_do: bool,
}

/// Join a bundle-relative path onto `dest`, rejecting anything that escapes it.
pub fn safe_join(dest: &Path, relative: &str) -> io::Result<PathBuf> {
    let relative_path = Path::new(relative);

    if relative_path.is_absolute() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("path must be relative: {relative}"),
        ));
    }

    for component in relative_path.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!("path escapes the destination: {relative}"),
            ));
        }
    }

    Ok(dest.join(relative_path))
}

fn bundle_path(dest: &Path, bundle: BundleId, relative: &str) -> io::Result<PathBuf> {
    safe_join(dest, &format!("{}/{}", bundle.slug(), relative))
}

fn place(
    path: PathBuf,
    bytes: &[u8],
    force: bool,
    dry_run: bool,
    report: &mut InstallReport,
) -> io::Result<()> {
    let existing = match fs::read(&path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };

    if let Some(current) = existing.as_deref() {
        if current == bytes {
            report.unchanged.push(path);
            return Ok(());
        }
        if !force {
            report.skipped.push(path);
            return Ok(());
        }
    }

    if !dry_run {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, bytes)?;
    }

    report.written.push(path);
    Ok(())
}

/// Write the selected bundles into `options.dest`.
pub fn install(options: &InstallOptions) -> io::Result<InstallReport> {
    if options.bundles.is_empty() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "no bundles selected",
        ));
    }

    let mut report = InstallReport {
        bundles: options.bundles.clone(),
        dry_run: options.dry_run,
        ..InstallReport::default()
    };

    for bundle in &options.bundles {
        for asset in assets::by_bundle(*bundle) {
            let path = bundle_path(&options.dest, *bundle, asset.path)?;
            place(path, asset.bytes, options.force, options.dry_run, &mut report)?;
        }

        for file in generated::for_bundle(*bundle) {
            let path = bundle_path(&options.dest, *bundle, file.path)?;
            place(
                path,
                file.contents.as_bytes(),
                options.force,
                options.dry_run,
                &mut report,
            )?;
        }
    }

    Ok(report)
}

/// Compare an installed tree against the given bundles.
pub fn verify(dest: &Path, bundles: &[BundleId]) -> io::Result<VerifyReport> {
    let mut report = VerifyReport {
        bundles: bundles.to_vec(),
        ..VerifyReport::default()
    };

    for bundle in bundles {
        for asset in assets::by_bundle(*bundle) {
            let path = bundle_path(dest, *bundle, asset.path)?;
            match fs::read(&path) {
                Ok(bytes) if bytes.as_slice() == asset.bytes => report.ok.push(path),
                Ok(_) => report.modified.push(path),
                Err(error) if error.kind() == ErrorKind::NotFound => report.missing.push(path),
                Err(error) => return Err(error),
            }
        }

        for file in generated::for_bundle(*bundle) {
            let path = bundle_path(dest, *bundle, file.path)?;
            match fs::read(&path) {
                Ok(bytes) if bytes == file.contents.as_bytes() => report.ok.push(path),
                Ok(_) => report.modified.push(path),
                Err(error) if error.kind() == ErrorKind::NotFound => report.missing.push(path),
                Err(error) => return Err(error),
            }
        }
    }

    Ok(report)
}

/// Move a v0.1.x flat EGM install (`<dest>/plugin.json`, `<dest>/skills/...`)
/// into the bundle layout (`<dest>/egm/...`).
///
/// Only files that match the embedded copy byte for byte are moved. Anything you
/// edited stays exactly where it is and is reported in [`MigrateReport::left`],
/// so this never destroys local work.
pub fn migrate(dest: &Path) -> io::Result<MigrateReport> {
    let bundle_root = dest.join(BundleId::Egm.slug());
    if bundle_root.exists() {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!(
                "{} already contains a '{}' directory; nothing to migrate",
                dest.display(),
                BundleId::Egm.slug()
            ),
        ));
    }

    let mut report = MigrateReport::default();

    for asset in assets::by_bundle(BundleId::Egm) {
        let old = safe_join(dest, asset.path)?;
        let new = bundle_path(dest, BundleId::Egm, asset.path)?;

        let existing = match fs::read(&old) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == ErrorKind::NotFound => None,
            Err(error) => return Err(error),
        };

        match existing {
            None => continue,
            Some(bytes) if bytes.as_slice() == asset.bytes => {
                if let Some(parent) = new.parent() {
                    fs::create_dir_all(parent)?;
                }
                if fs::rename(&old, &new).is_err() {
                    fs::copy(&old, &new)?;
                    fs::remove_file(&old)?;
                }
                report.moved.push((old, new));
            }
            Some(_) => report.left.push(old),
        }
    }

    report.nothing_to_do = report.moved.is_empty() && report.left.is_empty();
    Ok(report)
}
