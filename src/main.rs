// Copyright 2026 sufiyan-sabeel
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0

//! `indus-egm` — install agent bundles (EGM, REA) into a coding agent.

use std::path::PathBuf;
use std::process::{Command as ProcessCommand, ExitCode};

use clap::{Parser, Subcommand};

use indus_egm::{
    assets, generated, install, migrate, verify, BundleId, InstallOptions, Runtime, Target,
};

#[derive(Debug, Parser)]
#[command(
    name = "indus-egm",
    version,
    about = "Install agent bundles (Elephant-Goldfish Model, REA) into a coding agent",
    after_help = "Bundles:\n  egm   Elephant-Goldfish Model: spec drafting + fresh-context review\n  rea   REA: Reverse Engineer Anything: skills + MCP server registration\n\nExamples:\n  indus-egm install --target induscode\n  indus-egm install --bundle rea --dest ~/agent/plugins\n  indus-egm verify --bundle egm --target claude-code\n  indus-egm migrate --dest ~/.induscode/plugins/egm"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Copy the embedded bundles into a directory.
    Install {
        /// Target harness: claude-code, codex, antigravity, induscode.
        #[arg(long, value_name = "TARGET", conflicts_with = "dest")]
        target: Option<String>,
        /// Explicit destination directory; overrides --target.
        #[arg(long, value_name = "DIR")]
        dest: Option<PathBuf>,
        /// Bundles to install: egm, rea, or all.
        #[arg(long, value_name = "LIST", default_value = "all")]
        bundle: String,
        /// Overwrite files that exist but differ from the embedded copy.
        #[arg(long)]
        force: bool,
        /// Report what would be written without touching the filesystem.
        #[arg(long)]
        dry_run: bool,
    },
    /// Check whether an installed copy still matches the embedded bundles.
    Verify {
        /// Target harness: claude-code, codex, antigravity, induscode.
        #[arg(long, value_name = "TARGET", conflicts_with = "dest")]
        target: Option<String>,
        /// Explicit directory to check; overrides --target.
        #[arg(long, value_name = "DIR")]
        dest: Option<PathBuf>,
        /// Bundles to check: egm, rea, or all.
        #[arg(long, value_name = "LIST", default_value = "all")]
        bundle: String,
    },
    /// List the embedded files.
    List {
        /// Include byte sizes.
        #[arg(long)]
        sizes: bool,
        /// Bundles to list: egm, rea, or all.
        #[arg(long, value_name = "LIST", default_value = "all")]
        bundle: String,
    },
    /// Describe the available bundles, their licences and runtime needs.
    Bundles,
    /// Check this machine for the runtimes the bundles need.
    Doctor {
        /// Exit non-zero when a check fails.
        #[arg(long)]
        strict: bool,
    },
    /// Move a v0.1.x flat EGM install into the bundle layout.
    Migrate {
        /// Directory holding the old flat install; overrides --target.
        #[arg(long, value_name = "DIR")]
        dest: Option<PathBuf>,
    },
    /// Print the default install directory for each supported target.
    Paths,
}

fn home_dir() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .ok_or_else(|| "HOME is not set; pass --dest explicitly".to_string())
}

fn resolve_dest(target: Option<&str>, dest: Option<PathBuf>) -> Result<(String, PathBuf), String> {
    if let Some(directory) = dest {
        return Ok(("custom".to_string(), directory));
    }

    let name = target.unwrap_or("induscode");
    let parsed = Target::parse(name).ok_or_else(|| {
        format!(
            "unknown target '{name}'; expected one of: {}",
            Target::slugs().join(", ")
        )
    })?;

    let home = home_dir()?;
    Ok((parsed.slug().to_string(), parsed.default_dir(&home)))
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode, String> {
    match cli.command {
        Command::Install {
            target,
            dest,
            bundle,
            force,
            dry_run,
        } => {
            let bundles = BundleId::parse_list(&bundle)?;
            let (label, directory) = resolve_dest(target.as_deref(), dest)?;

            let options = InstallOptions::new(directory.clone())
                .bundles(bundles.clone())
                .force(force)
                .dry_run(dry_run);

            let report = install(&options).map_err(|error| {
                format!("could not install into {}: {error}", directory.display())
            })?;

            println!("target:  {label}");
            println!("dest:    {}", directory.display());
            for id in &bundles {
                println!(
                    "bundle:  {} {} ({} files)",
                    id.slug(),
                    id.version(),
                    assets::count_in(*id) + generated::for_bundle(*id).len()
                );
            }
            println!("result:  {}", report.summary());

            if report.dry_run {
                for path in &report.written {
                    println!("  would write {}", path.display());
                }
            }

            if !report.skipped.is_empty() {
                println!();
                println!(
                    "{} local file(s) differ and were left alone; re-run with --force to overwrite:",
                    report.skipped.len()
                );
                for path in &report.skipped {
                    println!("  {}", path.display());
                }
            }

            for id in &bundles {
                if let Some(runtime) = id.runtime() {
                    let (line, _) = check_runtime(runtime);
                    println!();
                    println!("runtime for {}: {line}", id.slug());
                    println!("  {}", runtime.note);
                }
            }

            if report.written.is_empty() && report.skipped.is_empty() {
                println!();
                println!("Already up to date.");
            }

            Ok(ExitCode::SUCCESS)
        }

        Command::Verify {
            target,
            dest,
            bundle,
        } => {
            let bundles = BundleId::parse_list(&bundle)?;
            let (label, directory) = resolve_dest(target.as_deref(), dest)?;

            let report = verify(&directory, &bundles)
                .map_err(|error| format!("could not verify {}: {error}", directory.display()))?;

            println!("target:  {label}");
            println!("dest:    {}", directory.display());
            println!(
                "result:  {} matching, {} missing, {} modified",
                report.ok.len(),
                report.missing.len(),
                report.modified.len()
            );

            for path in &report.missing {
                println!("  missing  {}", path.display());
            }
            for path in &report.modified {
                println!("  modified {}", path.display());
            }

            if report.is_clean() {
                Ok(ExitCode::SUCCESS)
            } else {
                Ok(ExitCode::from(2))
            }
        }

        Command::List { sizes, bundle } => {
            let bundles = BundleId::parse_list(&bundle)?;
            for id in &bundles {
                println!(
                    "{} - {} {} (upstream {} at {})",
                    id.slug(),
                    id.name(),
                    id.version(),
                    id.upstream(),
                    &id.commit()[..12]
                );
                for asset in assets::by_bundle(*id) {
                    if sizes {
                        println!("{:>8}  {}/{}", asset.bytes.len(), id.slug(), asset.path);
                    } else {
                        println!("{}/{}", id.slug(), asset.path);
                    }
                }
                for file in generated::for_bundle(*id) {
                    if sizes {
                        println!(
                            "{:>8}  {}/{} (generated)",
                            file.contents.len(),
                            id.slug(),
                            file.path
                        );
                    } else {
                        println!("{}/{} (generated)", id.slug(), file.path);
                    }
                }
            }
            Ok(ExitCode::SUCCESS)
        }

        Command::Bundles => {
            println!("indus-egm {}", env!("CARGO_PKG_VERSION"));
            for id in BundleId::ALL {
                println!();
                println!("{} — {}", id.slug(), id.name());
                println!("  {}", id.description());
                println!("  version:   {}", id.version());
                println!("  commit:    {}", id.commit());
                println!("  license:   {}", id.license());
                println!("  upstream:  {}", id.upstream());
                println!(
                    "  files:     {} embedded + {} generated",
                    assets::count_in(id),
                    generated::for_bundle(id).len()
                );
                match id.runtime() {
                    None => println!("  runtime:   none required"),
                    Some(runtime) => {
                        println!("  runtime:   {} {}", runtime.program, runtime.requirement);
                    }
                }
            }
            Ok(ExitCode::SUCCESS)
        }

        Command::Doctor { strict } => {
            let mut failures = 0usize;

            println!("indus-egm {}", env!("CARGO_PKG_VERSION"));
            match std::env::var_os("HOME") {
                Some(home) => println!("home:     {}", PathBuf::from(home).display()),
                None => {
                    println!("home:     NOT SET (pass --dest explicitly)");
                    failures += 1;
                }
            }

            for id in BundleId::ALL {
                println!();
                println!("{} — {}", id.slug(), id.name());
                println!(
                    "  files:    {} embedded + {} generated",
                    assets::count_in(id),
                    generated::for_bundle(id).len()
                );
                match id.runtime() {
                    None => println!("  runtime:  none required"),
                    Some(runtime) => {
                        println!("  needs:    {} {}", runtime.program, runtime.requirement);
                        let (line, ok) = check_runtime(runtime);
                        println!("  status:   {line}");
                        println!("  note:     {}", runtime.note);
                        if !ok {
                            failures += 1;
                        }
                    }
                }
            }

            println!();
            if failures == 0 {
                println!("all checks passed");
                Ok(ExitCode::SUCCESS)
            } else {
                println!("{failures} check(s) failed");
                if strict {
                    Ok(ExitCode::from(3))
                } else {
                    Ok(ExitCode::SUCCESS)
                }
            }
        }

        Command::Migrate { dest } => {
            let (label, directory) = resolve_dest(None, dest)?;
            let report = migrate(&directory)
                .map_err(|error| format!("could not migrate {}: {error}", directory.display()))?;

            println!("target:  {label}");
            println!("dest:    {}", directory.display());

            if report.nothing_to_do {
                println!("result:  nothing found at the v0.1.x layout");
                return Ok(ExitCode::SUCCESS);
            }

            println!(
                "result:  moved {} file(s) into {}/, left {} in place",
                report.moved.len(),
                BundleId::Egm.slug(),
                report.left.len()
            );

            for (from, to) in &report.moved {
                println!("  moved {} -> {}", from.display(), to.display());
            }
            for path in &report.left {
                println!("  left  {} (differs from the embedded copy)", path.display());
            }

            Ok(ExitCode::SUCCESS)
        }

        Command::Paths => {
            let home = home_dir()?;
            for target in Target::ALL {
                println!(
                    "{:<12} {}",
                    target.slug(),
                    target.default_dir(&home).display()
                );
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

/// Parse a `major.minor.patch` version out of something like `v22.23.4`.
fn parse_version(raw: &str) -> Option<(u64, u64, u64)> {
    let trimmed = raw.trim().trim_start_matches('v');
    let mut parts = trimmed.split('.');

    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next().unwrap_or("0");
    let patch = patch
        .split(|character| character == '-' || character == '+')
        .next()?
        .parse()
        .ok()?;

    Some((major, minor, patch))
}

/// Whether a Node version satisfies `^22.23.4 || ^24.22.1 || >=26.12.1`.
///
/// That range is REA's published `engines` field: it allows 22.23.4+, 24.22.1+
/// and 26.12.1+, and excludes the odd-numbered 23 and 25 releases.
fn node_satisfies(version: (u64, u64, u64)) -> bool {
    let (major, minor, patch) = version;

    match major {
        22 => (minor, patch) >= (23, 4),
        24 => (minor, patch) >= (22, 1),
        26 => (minor, patch) >= (12, 1),
        other => other > 26,
    }
}

fn check_runtime(runtime: Runtime) -> (String, bool) {
    match ProcessCommand::new(runtime.program).arg("--version").output() {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            match parse_version(&text) {
                Some(version) => {
                    let ok = node_satisfies(version);
                    let verdict = if ok { "satisfies" } else { "does not satisfy" };
                    (
                        format!(
                            "{} {} {verdict} {}",
                            runtime.program, text, runtime.requirement
                        ),
                        ok,
                    )
                }
                None => (
                    format!("{} {} (version not understood)", runtime.program, text),
                    false,
                ),
            }
        }
        Ok(_) => (
            format!("{} found but exited with an error", runtime.program),
            false,
        ),
        Err(_) => (format!("{} not found on PATH", runtime.program), false),
    }
}
