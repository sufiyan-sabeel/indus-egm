// Copyright 2026 sufiyan-sabeel
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0

//! The embedded bundles.
//!
//! Paths are relative to a bundle's root and use `/` separators, so they can be
//! joined onto `<dest>/<bundle>` for any destination directory.

use crate::bundle::BundleId;

/// One vendored file, baked into the binary at compile time.
#[derive(Debug, Clone, Copy)]
pub struct Asset {
    /// Bundle this file belongs to.
    pub bundle: BundleId,
    /// Path relative to the bundle root, e.g. `skills/egm-review/SKILL.md`.
    pub path: &'static str,
    /// Raw file contents.
    pub bytes: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/assets_generated.rs"));

/// Every embedded file, grouped by bundle and sorted by path.
pub fn all() -> &'static [Asset] {
    ASSETS
}

/// Embedded files belonging to one bundle.
pub fn by_bundle(bundle: BundleId) -> impl Iterator<Item = &'static Asset> {
    ASSETS.iter().filter(move |asset| asset.bundle == bundle)
}

/// Number of embedded files across all bundles.
pub fn count() -> usize {
    ASSETS.len()
}

/// Number of embedded files in one bundle.
pub fn count_in(bundle: BundleId) -> usize {
    by_bundle(bundle).count()
}

/// Look up a single file by bundle and bundle-relative path.
pub fn get(bundle: BundleId, path: &str) -> Option<&'static Asset> {
    by_bundle(bundle).find(|asset| asset.path == path)
}

/// Read an embedded file as UTF-8 text.
pub fn text(bundle: BundleId, path: &str) -> Option<&'static str> {
    get(bundle, path).and_then(|asset| std::str::from_utf8(asset.bytes).ok())
}

/// Names of the skills in a bundle, e.g. `egm-review` or `reverse-engineer-anything`.
pub fn skill_names(bundle: BundleId) -> Vec<&'static str> {
    let mut names: Vec<&'static str> = by_bundle(bundle)
        .filter_map(|asset| asset.path.strip_prefix("skills/"))
        .filter_map(|rest| rest.split_once('/').map(|(name, _)| name))
        .collect();
    names.sort_unstable();
    names.dedup();
    names
}

/// Names of the reviewer prompts in a bundle, e.g. `design_critic`.
pub fn agent_names(bundle: BundleId) -> Vec<&'static str> {
    let mut names: Vec<&'static str> = by_bundle(bundle)
        .filter_map(|asset| asset.path.strip_prefix("agents/"))
        .filter_map(|rest| rest.strip_suffix(".md"))
        .collect();
    names.sort_unstable();
    names
}
