// Copyright 2026 sufiyan-sabeel
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0

//! Copying the embedded bundle onto disk, and checking what landed there.

use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Component, Path, PathBuf};

use crate::assets;

/// How to install the bundle.
#[derive(Debug, Clone)]
pub struct InstallOptions {
    /// Directory the bundle root is written into.
    pub dest: PathBuf,
    /// Overwrite files that exist but differ from the embedded copy.
    pub force: bool,
    /// Report what would happen without touching the filesystem.
    pub dry_run: bool,
}

impl InstallOptions {
    /// Options for installing into `dest`, without force and without dry run.
    pub fn new(dest: impl Into<PathBuf>) -> Self {
        Self {
            dest: dest.into(),
            force: false,
            dry_run: false,
        }
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
    /// Files written (or that would be written, on a dry run).
    pub written: Vec<PathBuf>,
    /// Files already identical to the embedded copy.
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
            "{verb} {}, unchanged {}, skipped {}",
            self.written.len(),
            self.unchanged.len(),
            self.skipped.len()
        )
    }
}

/// Result of comparing an installed tree against the embedded bundle.
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
    /// Files that match.
    pub ok: Vec<PathBuf>,
    /// Files that are absent.
    pub missing: Vec<PathBuf>,
    /// Files that differ from the embedded copy.
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

/// Join a bundle-relative asset path onto `dest`, rejecting anything that escapes it.
pub fn safe_join(dest: &Path, relative: &str) -> io::Result<PathBuf> {
    let relative_path = Path::new(relative);

    if relative_path.is_absolute() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("asset path must be relative: {relative}"),
        ));
    }

    for component in relative_path.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!("asset path escapes the destination: {relative}"),
            ));
        }
    }

    Ok(dest.join(relative_path))
}

/// Write the embedded bundle into `options.dest`.
pub fn install(options: &InstallOptions) -> io::Result<InstallReport> {
    let mut report = InstallReport {
        dry_run: options.dry_run,
        ..InstallReport::default()
    };

    for asset in assets::all() {
        let path = safe_join(&options.dest, asset.path)?;

        let existing = match fs::read(&path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == ErrorKind::NotFound => None,
            Err(error) => return Err(error),
        };

        if let Some(current) = existing.as_deref() {
            if current == asset.bytes {
                report.unchanged.push(path);
                continue;
            }
            if !options.force {
                report.skipped.push(path);
                continue;
            }
        }

        if !options.dry_run {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, asset.bytes)?;
        }

        report.written.push(path);
    }

    Ok(report)
}

/// Compare an installed tree against the embedded bundle.
pub fn verify(dest: &Path) -> io::Result<VerifyReport> {
    let mut report = VerifyReport::default();

    for asset in assets::all() {
        let path = safe_join(dest, asset.path)?;

        match fs::read(&path) {
            Ok(bytes) if bytes.as_slice() == asset.bytes => report.ok.push(path),
            Ok(_) => report.modified.push(path),
            Err(error) if error.kind() == ErrorKind::NotFound => report.missing.push(path),
            Err(error) => return Err(error),
        }
    }

    Ok(report)
}
