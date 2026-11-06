// Copyright 2026 sufiyan-sabeel
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0

//! Install the [Elephant-Goldfish Model](https://egm.dev) (EGM) into a coding agent.
//!
//! EGM is a design-review method published by Google as a set of agent skills and
//! reviewer prompts: an *elephant* drafts a specification with full project context,
//! then fresh *goldfish* reviewers stress-test that draft with no memory of the
//! conversation. This crate embeds that bundle at compile time and copies it into a
//! directory your agent can read.
//!
//! ```no_run
//! use indus_egm::{install, verify, InstallOptions};
//!
//! let report = install(&InstallOptions::new("/home/me/.induscode/plugins/egm"))?;
//! println!("{} files written", report.written.len());
//!
//! assert!(verify(std::path::Path::new("/home/me/.induscode/plugins/egm"))?.is_clean());
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! The crate ships no model weights: EGM runs on whatever model your host agent
//! already uses. It is an installer and a bundle, not an inference engine.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod assets;
pub mod install;
pub mod target;

pub use assets::{all as all_assets, count as asset_count, get as get_asset, Asset};
pub use install::{
    install, verify, InstallOptions, InstallReport, VerifyReport, VerifyStatus,
};
pub use target::Target;

/// Version of the EGM bundle vendored into this release.
pub const EGM_VERSION: &str = "0.1.0";

/// Upstream commit of `google/elephant-goldfish-model` that `vendor/egm` was taken from.
pub const EGM_COMMIT: &str = "6a8001ea31ccf72430ba0603a467ec6df9f69dd7";

/// Upstream repository for the vendored bundle.
pub const EGM_REPOSITORY: &str = "https://github.com/google/elephant-goldfish-model";

/// License of the vendored bundle and of this crate.
pub const LICENSE: &str = "Apache-2.0";
