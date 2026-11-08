// Copyright 2026 sufiyan-sabeel
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0

//! Install agent bundles into a coding agent.
//!
//! Two bundles ship in one binary:
//!
//! - [`BundleId::Egm`] — Google's [Elephant-Goldfish Model](https://egm.dev): an
//!   *elephant* drafts a specification with full project context, then fresh
//!   *goldfish* reviewers stress-test it with no memory of the conversation.
//! - [`BundleId::Rea`] — [REA: Reverse Engineer Anything](https://rea.tools/): agent
//!   skills for binary, application and runtime analysis, plus an MCP server
//!   registration. Only REA's skills are embedded; its MCP server and native
//!   bridges come from npm at run time, so this crate stays small.
//!
//! Each bundle installs into its own directory under the destination:
//!
//! ```text
//! <dest>/egm/{plugin.json, skills/, agents/, resources/}
//! <dest>/rea/{skills/, LICENSE, SOURCE.json, mcp.json}
//! ```
//!
//! ```no_run
//! use indus_egm::{install, verify, BundleId, InstallOptions};
//!
//! let options = InstallOptions::new("/home/me/.induscode/plugins")
//!     .bundles(vec![BundleId::Egm, BundleId::Rea]);
//! let report = install(&options)?;
//! println!("{}", report.summary());
//!
//! assert!(verify("/home/me/.induscode/plugins", &[BundleId::Egm])?.is_clean());
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! Neither bundle contains model weights: they run on whatever model your host
//! agent already uses. This crate installs files, it does not run inference.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod assets;
pub mod bundle;
pub mod generated;
pub mod install;
pub mod target;

pub use assets::{all as all_assets, count as asset_count, get as get_asset, Asset};
pub use bundle::{BundleId, Runtime};
pub use generated::GeneratedFile;
pub use install::{
    install, migrate, verify, InstallOptions, InstallReport, MigrateReport, VerifyReport,
    VerifyStatus,
};
pub use target::Target;

/// License of this crate and of the EGM bundle.
pub const LICENSE: &str = "Apache-2.0";

/// Version of the vendored Elephant-Goldfish bundle.
pub const EGM_VERSION: &str = "0.1.0";

/// Upstream commit of `google/elephant-goldfish-model` that `vendor/egm` was taken from.
pub const EGM_COMMIT: &str = "6a8001ea31ccf72430ba0603a467ec6df9f69dd7";

/// Upstream repository of the EGM bundle.
pub const EGM_REPOSITORY: &str = "https://github.com/google/elephant-goldfish-model";

/// License of the vendored EGM bundle.
pub const EGM_LICENSE: &str = "Apache-2.0";

/// Version of the vendored REA bundle.
pub const REA_VERSION: &str = "6.8.0";

/// Upstream commit of `morluto/rea` that `vendor/rea` was taken from.
pub const REA_COMMIT: &str = "72d5e72e6d7b7997c4ecb11972fc405be8ec67d2";

/// Upstream repository of the REA bundle.
pub const REA_REPOSITORY: &str = "https://github.com/morluto/rea";

/// npm package that provides REA's MCP server and CLI.
pub const REA_NPM_PACKAGE: &str = "rea-agents";

/// Node range REA publishes in its `engines` field.
pub const REA_NODE_REQUIREMENT: &str = "^22.23.4 || ^24.22.1 || >=26.12.1";

/// License of the vendored REA bundle.
pub const REA_LICENSE: &str = "MIT";
