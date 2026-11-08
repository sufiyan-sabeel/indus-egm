// Copyright 2026 sufiyan-sabeel
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0

use std::path::Path;

use indus_egm::{
    assets, generated, install, migrate, verify, BundleId, InstallOptions, Target,
};

#[test]
fn both_bundles_are_embedded() {
    assert!(
        assets::count_in(BundleId::Egm) > 40,
        "expected the full EGM bundle, found {}",
        assets::count_in(BundleId::Egm)
    );
    assert_eq!(assets::count_in(BundleId::Rea), 18);
    assert_eq!(assets::count(), assets::count_in(BundleId::Egm) + assets::count_in(BundleId::Rea));
}

#[test]
fn egm_bundle_contains_its_pieces() {
    assert!(assets::get(BundleId::Egm, "plugin.json").is_some());
    assert!(assets::get(BundleId::Egm, "agents/design_critic.md").is_some());
    assert!(assets::get(BundleId::Egm, "resources/document_standards/feature-spec-standard.md").is_some());
    assert!(assets::text(BundleId::Egm, "skills/egm-review/SKILL.md").is_some());

    let skills = assets::skill_names(BundleId::Egm);
    for expected in [
        "egm-bootstrap",
        "egm-brainstorm",
        "egm-draft",
        "egm-drift-report",
        "egm-resolve",
        "egm-review",
        "egm-review-loop",
    ] {
        assert!(skills.contains(&expected), "missing EGM skill {expected}");
    }
    assert_eq!(assets::agent_names(BundleId::Egm).len(), 10);
}

#[test]
fn rea_bundle_contains_its_pieces() {
    let skills = assets::skill_names(BundleId::Rea);
    for expected in [
        "rea-analysis-optimization",
        "rea-changelog-update",
        "rea-ci-tests-audit",
        "rea-tool-design",
        "reverse-engineer-anything",
    ] {
        assert!(skills.contains(&expected), "missing REA skill {expected}");
    }

    assert!(assets::get(BundleId::Rea, "LICENSE").is_some());
    assert!(assets::get(BundleId::Rea, "SOURCE.json").is_some());
    assert!(assets::text(BundleId::Rea, "skills/reverse-engineer-anything/SKILL.md").is_some());

    // The heavy runtime must never be vendored.
    assert!(assets::get(BundleId::Rea, "src/index.ts").is_none());
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
fn rea_mcp_registration_is_deterministic() {
    let first = generated::rea_mcp_json();
    assert_eq!(first, generated::rea_mcp_json());
    assert!(first.contains("\"command\": \"npx\""));
    assert!(first.contains("rea-agents@6.8.0"));
    assert!(first.contains("\"mcp\""));
    assert!(first.starts_with("{\n"));
}

#[test]
fn default_set_installs_both_bundles() {
    assert_eq!(BundleId::default_set(), vec![BundleId::Egm, BundleId::Rea]);
}

#[test]
fn bundle_lists_parse() {
    assert_eq!(BundleId::parse_list("egm").unwrap(), vec![BundleId::Egm]);
    assert_eq!(BundleId::parse_list("all").unwrap(), vec![BundleId::Egm, BundleId::Rea]);
    assert_eq!(BundleId::parse_list("").unwrap(), vec![BundleId::Egm, BundleId::Rea]);
    assert_eq!(
        BundleId::parse_list("rea, egm").unwrap(),
        vec![BundleId::Egm, BundleId::Rea]
    );
    assert_eq!(BundleId::parse_list("rea,rea").unwrap(), vec![BundleId::Rea]);
    assert!(BundleId::parse_list("nope").is_err());
}

#[test]
fn install_writes_the_bundle_layout() {
    let directory = tempfile::tempdir().unwrap();
    let report = install(&InstallOptions::new(directory.path())).unwrap();

    let expected = assets::count() + generated::count_in(&BundleId::default_set());
    assert_eq!(report.written.len(), expected);
    assert!(directory.path().join("egm/plugin.json").is_file());
    assert!(directory.path().join("egm/skills/egm-review/SKILL.md").is_file());
    assert!(directory.path().join("rea/mcp.json").is_file());
    assert!(directory.path().join("rea/skills/reverse-engineer-anything/SKILL.md").is_file());

    // Nothing is written at the destination root any more.
    assert!(!directory.path().join("plugin.json").exists());
}

#[test]
fn install_then_verify_roundtrip() {
    let directory = tempfile::tempdir().unwrap();
    install(&InstallOptions::new(directory.path())).unwrap();

    let check = verify(directory.path(), &BundleId::default_set()).unwrap();
    assert!(check.is_clean(), "unexpected differences: {:?}", check.missing);
    assert_eq!(
        check.ok.len(),
        assets::count() + generated::count_in(&BundleId::default_set())
    );
}

#[test]
fn installing_one_bundle_leaves_the_other_absent() {
    let directory = tempfile::tempdir().unwrap();
    let report = install(&InstallOptions::new(directory.path()).bundles(vec![BundleId::Rea])).unwrap();

    assert_eq!(report.bundles, vec![BundleId::Rea]);
    assert!(directory.path().join("rea/mcp.json").is_file());
    assert!(!directory.path().join("egm").exists());

    let check = verify(directory.path(), &[BundleId::Rea]).unwrap();
    assert!(check.is_clean());
}

#[test]
fn install_is_idempotent() {
    let directory = tempfile::tempdir().unwrap();
    install(&InstallOptions::new(directory.path())).unwrap();

    let second = install(&InstallOptions::new(directory.path())).unwrap();
    assert!(second.written.is_empty());
    assert_eq!(second.unchanged.len(), second.total());
}

#[test]
fn local_edits_survive_without_force() {
    let directory = tempfile::tempdir().unwrap();
    install(&InstallOptions::new(directory.path())).unwrap();

    let edited = directory.path().join("egm/agents/design_critic.md");
    std::fs::write(&edited, b"local change").unwrap();

    let report = install(&InstallOptions::new(directory.path())).unwrap();
    assert_eq!(report.skipped, vec![edited.clone()]);
    assert_eq!(std::fs::read_to_string(&edited).unwrap(), "local change");

    let check = verify(directory.path(), &[BundleId::Egm]).unwrap();
    assert!(!check.is_clean());
    assert_eq!(check.modified, vec![edited]);
}

#[test]
fn force_overwrites_local_edits() {
    let directory = tempfile::tempdir().unwrap();
    install(&InstallOptions::new(directory.path())).unwrap();

    let edited = directory.path().join("rea/mcp.json");
    std::fs::write(&edited, b"{}").unwrap();

    let report = install(&InstallOptions::new(directory.path()).force(true)).unwrap();
    assert_eq!(report.written, vec![edited.clone()]);
    assert_eq!(
        std::fs::read_to_string(&edited).unwrap(),
        generated::rea_mcp_json()
    );
}

#[test]
fn dry_run_writes_nothing() {
    let directory = tempfile::tempdir().unwrap();
    let report = install(&InstallOptions::new(directory.path()).dry_run(true)).unwrap();

    assert!(report.dry_run);
    assert_eq!(report.written.len(), report.total());
    assert!(!directory.path().join("egm").exists());
    assert!(!directory.path().join("rea").exists());
}

#[test]
fn verify_reports_missing_files() {
    let directory = tempfile::tempdir().unwrap();
    let check = verify(directory.path(), &[BundleId::Rea]).unwrap();

    assert!(!check.is_clean());
    assert_eq!(check.missing.len(), assets::count_in(BundleId::Rea) + 1);
    assert!(check.ok.is_empty());
}

#[test]
fn empty_bundle_selection_is_rejected() {
    let directory = tempfile::tempdir().unwrap();
    let error = install(&InstallOptions::new(directory.path()).bundles(vec![])).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
}

#[test]
fn migrate_moves_a_flat_v01_install() {
    let directory = tempfile::tempdir().unwrap();

    // Reproduce the v0.1.x layout: EGM files at the destination root.
    for asset in assets::by_bundle(BundleId::Egm) {
        let path = directory.path().join(asset.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, asset.bytes).unwrap();
    }
    assert!(directory.path().join("plugin.json").is_file());

    let report = migrate(directory.path()).unwrap();
    assert_eq!(report.moved.len(), assets::count_in(BundleId::Egm));
    assert!(report.left.is_empty());
    assert!(!report.nothing_to_do);

    assert!(directory.path().join("egm/plugin.json").is_file());
    assert!(!directory.path().join("plugin.json").exists());
    assert!(verify(directory.path(), &[BundleId::Egm]).unwrap().is_clean());
}

#[test]
fn migrate_leaves_edited_files_alone() {
    let directory = tempfile::tempdir().unwrap();

    for asset in assets::by_bundle(BundleId::Egm) {
        let path = directory.path().join(asset.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, asset.bytes).unwrap();
    }
    let edited = directory.path().join("agents/design_critic.md");
    std::fs::write(&edited, b"mine").unwrap();

    let report = migrate(directory.path()).unwrap();
    assert_eq!(report.moved.len(), assets::count_in(BundleId::Egm) - 1);
    assert_eq!(report.left, vec![edited.clone()]);
    assert_eq!(std::fs::read_to_string(&edited).unwrap(), "mine");
}

#[test]
fn migrate_refuses_a_tree_that_is_already_migrated() {
    let directory = tempfile::tempdir().unwrap();
    install(&InstallOptions::new(directory.path()).bundles(vec![BundleId::Egm])).unwrap();

    let error = migrate(directory.path()).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
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
    assert_eq!(Target::slugs().len(), 4);
}
