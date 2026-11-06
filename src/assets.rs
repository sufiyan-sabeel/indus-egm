// Copyright 2026 sufiyan-sabeel
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0

//! The embedded EGM bundle.
//!
//! Paths are always relative to the bundle root and use `/` separators, so they
//! can be joined onto any destination directory.

/// One vendored file, baked into the binary at compile time.
#[derive(Debug, Clone, Copy)]
pub struct Asset {
    /// Bundle-relative path, e.g. `skills/egm-review/SKILL.md`.
    pub path: &'static str,
    /// Raw file contents.
    pub bytes: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/assets_generated.rs"));

/// Every embedded file, sorted by path.
pub fn all() -> &'static [Asset] {
    ASSETS
}

/// Number of embedded files.
pub fn count() -> usize {
    ASSETS.len()
}

/// Look up a single file by bundle-relative path.
pub fn get(path: &str) -> Option<&'static Asset> {
    ASSETS.iter().find(|asset| asset.path == path)
}

/// Read an embedded file as UTF-8 text.
pub fn text(path: &str) -> Option<&'static str> {
    get(path).and_then(|asset| std::str::from_utf8(asset.bytes).ok())
}

/// Names of the embedded skills, e.g. `egm-review`.
pub fn skill_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = ASSETS
        .iter()
        .filter_map(|asset| asset.path.strip_prefix("skills/"))
        .filter_map(|rest| rest.split_once('/').map(|(name, _)| name))
        .collect();
    names.sort_unstable();
    names.dedup();
    names
}

/// Names of the embedded reviewer prompts, e.g. `design_critic`.
pub fn agent_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = ASSETS
        .iter()
        .filter_map(|asset| asset.path.strip_prefix("agents/"))
        .filter_map(|rest| rest.strip_suffix(".md"))
        .collect();
    names.sort_unstable();
    names
}
