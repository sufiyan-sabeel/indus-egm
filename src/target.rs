// Copyright 2026 sufiyan-sabeel
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0

//! Supported install targets and their default directories.
//!
//! These defaults are conveniences, not contracts: harnesses move their plugin
//! directories between releases. Pass an explicit destination when yours differs.

use std::fmt;
use std::path::{Path, PathBuf};

/// A coding agent that can host the EGM skills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    /// Claude Code.
    ClaudeCode,
    /// Codex CLI.
    Codex,
    /// Antigravity (`agy`).
    Antigravity,
    /// The `induscode` Rust coding agent (`indusr` / `indusagir`).
    IndusCode,
}

impl Target {
    /// Every supported target.
    pub const ALL: [Target; 4] = [
        Target::ClaudeCode,
        Target::Codex,
        Target::Antigravity,
        Target::IndusCode,
    ];

    /// Stable identifier used on the command line.
    pub fn slug(self) -> &'static str {
        match self {
            Target::ClaudeCode => "claude-code",
            Target::Codex => "codex",
            Target::Antigravity => "antigravity",
            Target::IndusCode => "induscode",
        }
    }

    /// Human-readable name.
    pub fn display_name(self) -> &'static str {
        match self {
            Target::ClaudeCode => "Claude Code",
            Target::Codex => "Codex CLI",
            Target::Antigravity => "Antigravity",
            Target::IndusCode => "induscode",
        }
    }

    /// Parse a target from a slug or one of its accepted aliases.
    pub fn parse(input: &str) -> Option<Target> {
        match input.trim().to_ascii_lowercase().as_str() {
            "claude-code" | "claude" | "claudecode" | "cc" => Some(Target::ClaudeCode),
            "codex" | "codex-cli" => Some(Target::Codex),
            "antigravity" | "agy" => Some(Target::Antigravity),
            "induscode" | "indus" | "indusr" | "indusagir" => Some(Target::IndusCode),
            _ => None,
        }
    }

    /// All slugs, for error messages and help text.
    pub fn slugs() -> Vec<&'static str> {
        Target::ALL.iter().map(|target| target.slug()).collect()
    }

    /// Default install directory for this target, given a home directory.
    pub fn default_dir(self, home: &Path) -> PathBuf {
        match self {
            Target::ClaudeCode => home.join(".claude/plugins/egm"),
            Target::Codex => home.join(".codex/plugins/egm"),
            Target::Antigravity => home.join(".antigravity/plugins/egm"),
            Target::IndusCode => home.join(".induscode/plugins/egm"),
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.slug())
    }
}
