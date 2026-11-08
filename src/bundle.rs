// Copyright 2026 sufiyan-sabeel
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0

//! The bundles this tool can install, and what each one needs at run time.

use std::fmt;

/// A set of agent skills shipped as one unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BundleId {
    /// Google's Elephant-Goldfish Model: specification drafting and fresh-context review.
    Egm,
    /// REA: Reverse Engineer Anything — agent skills plus an MCP server.
    Rea,
}

/// Something a bundle needs from the host machine.
#[derive(Debug, Clone, Copy)]
pub struct Runtime {
    /// Executable that must be on `PATH`.
    pub program: &'static str,
    /// Version range the executable must satisfy, as published upstream.
    pub requirement: &'static str,
    /// How to get a usable runtime.
    pub note: &'static str,
}

impl BundleId {
    /// Every supported bundle, in install order.
    pub const ALL: [BundleId; 2] = [BundleId::Egm, BundleId::Rea];

    /// Stable identifier used on the command line and as the install directory name.
    pub fn slug(self) -> &'static str {
        match self {
            BundleId::Egm => "egm",
            BundleId::Rea => "rea",
        }
    }

    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            BundleId::Egm => "Elephant-Goldfish Model",
            BundleId::Rea => "REA: Reverse Engineer Anything",
        }
    }

    /// One-line description.
    pub fn description(self) -> &'static str {
        match self {
            BundleId::Egm => "Draft specifications with full context, then review them with fresh subagents.",
            BundleId::Rea => "Reverse engineer binaries, applications and runtime behaviour through an MCP server.",
        }
    }

    /// License of the vendored files.
    pub fn license(self) -> &'static str {
        match self {
            BundleId::Egm => "Apache-2.0",
            BundleId::Rea => "MIT",
        }
    }

    /// Upstream repository.
    pub fn upstream(self) -> &'static str {
        match self {
            BundleId::Egm => crate::EGM_REPOSITORY,
            BundleId::Rea => crate::REA_REPOSITORY,
        }
    }

    /// Upstream version of the vendored files.
    pub fn version(self) -> &'static str {
        match self {
            BundleId::Egm => crate::EGM_VERSION,
            BundleId::Rea => crate::REA_VERSION,
        }
    }

    /// Upstream commit the vendored files were taken from.
    pub fn commit(self) -> &'static str {
        match self {
            BundleId::Egm => crate::EGM_COMMIT,
            BundleId::Rea => crate::REA_COMMIT,
        }
    }

    /// What this bundle needs on the host, if anything.
    pub fn runtime(self) -> Option<Runtime> {
        match self {
            BundleId::Egm => None,
            BundleId::Rea => Some(Runtime {
                program: "node",
                requirement: crate::REA_NODE_REQUIREMENT,
                note: "REA's MCP server is fetched from npm on first run; Node 22.23.4+ is required.",
            }),
        }
    }

    /// Parse a single bundle from a slug or alias.
    pub fn parse(input: &str) -> Option<BundleId> {
        match input.trim().to_ascii_lowercase().as_str() {
            "egm" | "elephant" | "goldfish" | "elephant-goldfish" => Some(BundleId::Egm),
            "rea" | "reverse-engineer-anything" => Some(BundleId::Rea),
            _ => None,
        }
    }

    /// Parse a comma-separated bundle list. Empty input and `all` select every bundle.
    pub fn parse_list(input: &str) -> Result<Vec<BundleId>, String> {
        let trimmed = input.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("all") || trimmed == "*" {
            return Ok(BundleId::default_set());
        }

        let mut selected: Vec<BundleId> = Vec::new();
        for part in trimmed.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            if part.eq_ignore_ascii_case("all") || part == "*" {
                return Ok(BundleId::default_set());
            }
            let bundle = BundleId::parse(part).ok_or_else(|| {
                format!(
                    "unknown bundle '{part}'; expected one of: {} (or 'all')",
                    BundleId::slugs().join(", ")
                )
            })?;
            if !selected.contains(&bundle) {
                selected.push(bundle);
            }
        }

        if selected.is_empty() {
            return Err("no bundles selected".to_string());
        }

        selected.sort();
        Ok(selected)
    }

    /// All slugs, for help text and error messages.
    pub fn slugs() -> Vec<&'static str> {
        BundleId::ALL.iter().map(|bundle| bundle.slug()).collect()
    }

    /// The default install set: every bundle.
    pub fn default_set() -> Vec<BundleId> {
        BundleId::ALL.to_vec()
    }
}

impl fmt::Display for BundleId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.slug())
    }
}
