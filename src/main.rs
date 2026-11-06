// Copyright 2026 sufiyan-sabeel
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0

//! `indus-egm` — install the Elephant-Goldfish Model into a coding agent.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use indus_egm::{assets, install, verify, InstallOptions, Target};

#[derive(Debug, Parser)]
#[command(
    name = "indus-egm",
    version,
    about = "Install the Elephant-Goldfish Model (EGM) design-review skills into a coding agent",
    after_help = "Example:\n  indus-egm install --target induscode\n  indus-egm install --dest ~/my-agent/skills/egm --force\n  indus-egm verify --target claude-code"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Copy the embedded EGM skills, reviewer prompts and standards into a directory.
    Install {
        /// Target harness: claude-code, codex, antigravity, induscode.
        #[arg(long, value_name = "TARGET", conflicts_with = "dest")]
        target: Option<String>,
        /// Explicit destination directory; overrides --target.
        #[arg(long, value_name = "DIR")]
        dest: Option<PathBuf>,
        /// Overwrite files that exist but differ from the embedded copy.
        #[arg(long)]
        force: bool,
        /// Report what would be written without touching the filesystem.
        #[arg(long)]
        dry_run: bool,
    },
    /// List the embedded EGM assets.
    List {
        /// Include byte sizes.
        #[arg(long)]
        sizes: bool,
    },
    /// Check whether an installed copy still matches the embedded assets.
    Verify {
        /// Target harness: claude-code, codex, antigravity, induscode.
        #[arg(long, value_name = "TARGET", conflicts_with = "dest")]
        target: Option<String>,
        /// Explicit directory to check; overrides --target.
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
            force,
            dry_run,
        } => {
            let (label, directory) = resolve_dest(target.as_deref(), dest)?;
            let options = InstallOptions::new(directory.clone())
                .force(force)
                .dry_run(dry_run);

            let report = install(&options)
                .map_err(|error| format!("could not install into {}: {error}", directory.display()))?;

            println!("target:  {label}");
            println!("dest:    {}", directory.display());
            println!("bundle:  EGM {} ({} files)", indus_egm::EGM_VERSION, assets::count());
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

            if report.written.is_empty() && report.skipped.is_empty() {
                println!();
                println!("Already up to date.");
            }

            Ok(ExitCode::SUCCESS)
        }

        Command::List { sizes } => {
            println!(
                "EGM {} - {} files (upstream commit {})",
                indus_egm::EGM_VERSION,
                assets::count(),
                &indus_egm::EGM_COMMIT[..12]
            );
            for asset in assets::all() {
                if sizes {
                    println!("{:>8}  {}", asset.bytes.len(), asset.path);
                } else {
                    println!("{}", asset.path);
                }
            }
            Ok(ExitCode::SUCCESS)
        }

        Command::Verify { target, dest } => {
            let (label, directory) = resolve_dest(target.as_deref(), dest)?;
            let report = verify(&directory)
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
