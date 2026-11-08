// Copyright 2026 sufiyan-sabeel
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0

//! Files generated at install time rather than vendored.
//!
//! These are deterministic functions of the crate's constants, so an installed
//! copy can be byte-compared against them by [`crate::verify`] just like a
//! vendored asset. Nothing here depends on the clock or the environment.

use crate::bundle::BundleId;
use crate::{REA_NPM_PACKAGE, REA_VERSION};

/// A file produced by the installer instead of being embedded.
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    /// Bundle the file belongs to.
    pub bundle: BundleId,
    /// Path relative to the bundle root.
    pub path: &'static str,
    /// Exact contents written to disk.
    pub contents: String,
}

/// Generated files for one bundle.
pub fn for_bundle(bundle: BundleId) -> Vec<GeneratedFile> {
    match bundle {
        BundleId::Egm => Vec::new(),
        BundleId::Rea => vec![GeneratedFile {
            bundle,
            path: "mcp.json",
            contents: rea_mcp_json(),
        }],
    }
}

/// Total number of generated files across the given bundles.
pub fn count_in(bundles: &[BundleId]) -> usize {
    bundles
        .iter()
        .map(|bundle| for_bundle(*bundle).len())
        .sum()
}

/// MCP registration for REA, in the `mcpServers` shape most agents accept.
///
/// The server is started through `npx`, pinned to the vendored version, so the
/// heavy TypeScript runtime is never copied into this crate.
pub fn rea_mcp_json() -> String {
    let mut json = String::new();
    json.push_str("{\n");
    json.push_str("  \"mcpServers\": {\n");
    json.push_str("    \"rea\": {\n");
    json.push_str("      \"command\": \"npx\",\n");
    json.push_str(&format!(
        "      \"args\": [\"-y\", \"{REA_NPM_PACKAGE}@{REA_VERSION}\", \"mcp\"],\n"
    ));
    json.push_str("      \"startup_timeout_sec\": 120\n");
    json.push_str("    }\n");
    json.push_str("  }\n");
    json.push_str("}\n");
    json
}
