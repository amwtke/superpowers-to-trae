//! End-to-end tests for `--addons ddd`.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn cli() -> Command {
    Command::cargo_bin("superpowers-trae").unwrap()
}

#[test]
fn init_addons_ddd_writes_ddd_skills() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--addons", "ddd"])
        .assert()
        .success();

    assert!(tmp.path().join(".trae/skills/ddd/ddd-storm/SKILL.md").exists());
    assert!(tmp.path().join(".trae/skills/ddd/ddd-model/SKILL.md").exists());
    assert!(tmp.path().join(".trae/skills/ddd/ddd-spec/SKILL.md").exists());
}

#[test]
fn init_addons_ddd_writes_domain_md() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--addons", "ddd"])
        .assert()
        .success();

    let body = fs::read_to_string(tmp.path().join("DOMAIN.md")).unwrap();
    assert!(body.contains("领域模型"));
    assert!(body.contains("Single Source of Truth"));
}

#[test]
fn init_addons_ddd_writes_readme_harness() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--addons", "ddd"])
        .assert()
        .success();

    assert!(tmp.path().join("README-DDD-HARNESS.md").exists());
}

#[test]
fn init_addons_ddd_extends_agents_md() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--addons", "ddd"])
        .assert()
        .success();

    let body = fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
    assert!(body.contains("AGENTS DIRECTIVE"));
    assert!(body.contains("DDD methodology"));
    assert!(body.contains("Ubiquitous Language"));
}

#[test]
fn upgrade_addons_ddd_preserves_user_domain_md() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--addons", "ddd"])
        .assert()
        .success();

    // User edits DOMAIN.md
    let domain = tmp.path().join("DOMAIN.md");
    fs::write(&domain, "USER MODEL CONTENT — should be preserved").unwrap();

    cli()
        .args(["upgrade", "--dir", tmp.path().to_str().unwrap(), "--addons", "ddd"])
        .assert()
        .success();

    let body = fs::read_to_string(&domain).unwrap();
    assert_eq!(body, "USER MODEL CONTENT — should be preserved");
}

#[test]
fn upgrade_without_addons_keeps_existing_ddd_files() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--addons", "ddd"])
        .assert()
        .success();

    // User modifies a ddd skill (simulating user changes; even though static files,
    // user might do this)
    let storm = tmp.path().join(".trae/skills/ddd/ddd-storm/SKILL.md");
    fs::write(&storm, "USER MODIFIED DDD SKILL").unwrap();

    // upgrade without --addons should not touch ddd files
    cli()
        .args(["upgrade", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    let body = fs::read_to_string(&storm).unwrap();
    assert_eq!(body, "USER MODIFIED DDD SKILL");
}
