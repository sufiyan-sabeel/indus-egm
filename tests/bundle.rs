// Copyright 2026 sufiyan-sabeel
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0

use std::path::Path;

use indus_egm::{assets, install, verify, InstallOptions, Target};

#[test]
fn bundle_contains_the_egm_pieces() {
    assert!(
        assets::count() > 40,
        "expected the full EGM bundle, found {} files",
        assets::count()
    );
    assert!(assets::get("plugin.json").is_some());
    assert!(assets::get("agents/design_critic.md").is_some());
    assert!(assets::get("resources/document_standards/feature-spec-standard.md").is_some());
    assert!(assets::text("skills/egm-review/SKILL.md").is_some());
}

#[test]
fn all_seven_skills_are_embedded() {
    let skills = assets::skill_names();
    for expected in [
        "egm-bootstrap",
        "egm-brainstorm",
        "egm-draft",
        "egm-drift-report",
        "egm-resolve",
        "egm-review",
        "egm-review-loop",
    ] {
        assert!(skills.contains(&expected), "missing skill {expected}");
    }
}

#[test]
fn asset_paths_are_relative_and_safe() {
    for asset in assets::all() {
        assert!(!asset.path.starts_with('/'), "absolute path: {}", asset.path);
        assert!(!asset.path.contains(".."), "traversal: {}", asset.path);
        assert!(!asset.path.contains('\\'), "backslash: {}", asset.path);
        assert!(!asset.bytes.is_empty(), "empty file: {}", asset.path);
    }
}

#[test]
fn install_then_verify_roundtrip() {
    let directory = tempfile::tempdir().unwrap();
    let report = install(&InstallOptions::new(directory.path()).force(true)).unwrap();

    assert_eq!(report.written.len(), assets::count());
    assert!(report.skipped.is_empty());

    let check = verify(directory.path()).unwrap();
    assert!(check.is_clean(), "unexpected differences: {:?}", check.missing);
    assert_eq!(check.ok.len(), assets::count());
}

#[test]
fn install_is_idempotent() {
    let directory = tempfile::tempdir().unwrap();
    install(&InstallOptions::new(directory.path()).force(true)).unwrap();

    let second = install(&InstallOptions::new(directory.path())).unwrap();
    assert!(second.written.is_empty());
    assert_eq!(second.unchanged.len(), assets::count());
}

#[test]
fn local_edits_survive_without_force() {
    let directory = tempfile::tempdir().unwrap();
    install(&InstallOptions::new(directory.path()).force(true)).unwrap();

    let edited = directory.path().join("agents/design_critic.md");
    std::fs::write(&edited, b"local change").unwrap();

    let report = install(&InstallOptions::new(directory.path())).unwrap();
    assert_eq!(report.skipped, vec![edited.clone()]);
    assert_eq!(std::fs::read_to_string(&edited).unwrap(), "local change");

    let check = verify(directory.path()).unwrap();
    assert!(!check.is_clean());
    assert_eq!(check.modified, vec![edited]);
}

#[test]
fn force_overwrites_local_edits() {
    let directory = tempfile::tempdir().unwrap();
    install(&InstallOptions::new(directory.path()).force(true)).unwrap();

    let edited = directory.path().join("agents/design_critic.md");
    std::fs::write(&edited, b"local change").unwrap();

    let report = install(&InstallOptions::new(directory.path()).force(true)).unwrap();
    assert_eq!(report.written, vec![edited.clone()]);
    assert_eq!(
        std::fs::read(&edited).unwrap(),
        assets::get("agents/design_critic.md").unwrap().bytes
    );
}

#[test]
fn dry_run_writes_nothing() {
    let directory = tempfile::tempdir().unwrap();
    let report = install(&InstallOptions::new(directory.path()).dry_run(true)).unwrap();

    assert!(report.dry_run);
    assert_eq!(report.written.len(), assets::count());
    assert!(!directory.path().join("plugin.json").exists());
}

#[test]
fn verify_reports_missing_files() {
    let directory = tempfile::tempdir().unwrap();
    let check = verify(directory.path()).unwrap();

    assert!(!check.is_clean());
    assert_eq!(check.missing.len(), assets::count());
    assert!(check.ok.is_empty());
}

#[test]
fn targets_parse_and_resolve_defaults() {
    let home = Path::new("/home/example");

    assert_eq!(Target::parse("claude-code"), Some(Target::ClaudeCode));
    assert_eq!(Target::parse("CLAUDE"), Some(Target::ClaudeCode));
    assert_eq!(Target::parse("agy"), Some(Target::Antigravity));
    assert_eq!(Target::parse("indusr"), Some(Target::IndusCode));
    assert_eq!(Target::parse("nope"), None);

    assert_eq!(
        Target::IndusCode.default_dir(home),
        home.join(".induscode/plugins/egm")
    );
    assert_eq!(
        Target::ClaudeCode.default_dir(home),
        home.join(".claude/plugins/egm")
    );
    assert_eq!(Target::slugs().len(), 4);
}
