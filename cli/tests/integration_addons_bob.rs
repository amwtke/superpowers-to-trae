//! End-to-end tests for `--addons bob`.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn cli() -> Command {
    Command::cargo_bin("superpowers-trae").unwrap()
}

#[test]
fn init_addons_bob_writes_bob_skills() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--addons", "bob"])
        .assert()
        .success();

    assert!(tmp.path().join(".trae/skills/bob/bob-identify/SKILL.md").exists());
    assert!(tmp.path().join(".trae/skills/bob/bob-onion/SKILL.md").exists());
    assert!(tmp.path().join(".trae/skills/bob/bob-spec/SKILL.md").exists());
}

#[test]
fn init_addons_bob_writes_bob_md() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--addons", "bob"])
        .assert()
        .success();

    let body = fs::read_to_string(tmp.path().join("BOB.md")).unwrap();
    assert!(body.contains("Bob 4 环"));
    assert!(body.contains("Single Source of Truth"));
}

#[test]
fn init_addons_bob_writes_readme_harness() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--addons", "bob"])
        .assert()
        .success();

    assert!(tmp.path().join("README-RUN-BOB.md").exists());
}

#[test]
fn init_addons_bob_extends_agents_md() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--addons", "bob"])
        .assert()
        .success();

    let body = fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
    assert!(body.contains("AGENTS DIRECTIVE"));
    assert!(body.contains("Bob 4 环 Clean Architecture"));
    assert!(body.contains("框架边界外推"));
}

#[test]
fn init_addons_bob_includes_bob_readme_in_gitignore() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--addons", "bob"])
        .assert()
        .success();

    let gitignore = fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
    assert!(gitignore.contains(".trae/"));
    assert!(gitignore.contains("README-RUN-BOB.md"));
    assert!(
        !gitignore.contains("README-DDD-HARNESS.md"),
        "DDD readme leaked when only bob installed"
    );
}

#[test]
fn upgrade_addons_bob_preserves_user_bob_md() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--addons", "bob"])
        .assert()
        .success();

    // User edits BOB.md
    let bob = tmp.path().join("BOB.md");
    fs::write(&bob, "USER ARCH CONTENT — should be preserved").unwrap();

    cli()
        .args(["upgrade", "--dir", tmp.path().to_str().unwrap(), "--addons", "bob"])
        .assert()
        .success();

    let body = fs::read_to_string(&bob).unwrap();
    assert_eq!(body, "USER ARCH CONTENT — should be preserved");
}

#[test]
fn upgrade_without_addons_keeps_existing_bob_files() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--addons", "bob"])
        .assert()
        .success();

    // User modifies a bob skill
    let identify = tmp.path().join(".trae/skills/bob/bob-identify/SKILL.md");
    fs::write(&identify, "USER MODIFIED BOB SKILL").unwrap();

    // upgrade without --addons should not touch bob files
    cli()
        .args(["upgrade", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    let body = fs::read_to_string(&identify).unwrap();
    assert_eq!(body, "USER MODIFIED BOB SKILL");
}

#[test]
fn init_addons_ddd_and_bob_writes_both() {
    let tmp = tempdir().unwrap();
    cli()
        .args([
            "init",
            "--dir",
            tmp.path().to_str().unwrap(),
            "--addons",
            "ddd,bob",
        ])
        .assert()
        .success();

    // ddd files
    assert!(tmp.path().join(".trae/skills/ddd/ddd-storm/SKILL.md").exists());
    assert!(tmp.path().join("DOMAIN.md").exists());
    assert!(tmp.path().join("README-DDD-HARNESS.md").exists());

    // bob files
    assert!(tmp.path().join(".trae/skills/bob/bob-identify/SKILL.md").exists());
    assert!(tmp.path().join("BOB.md").exists());
    assert!(tmp.path().join("README-RUN-BOB.md").exists());

    // gitignore lists both
    let gi = fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
    assert!(gi.contains("README-DDD-HARNESS.md"));
    assert!(gi.contains("README-RUN-BOB.md"));

    // AGENTS.md carries both segments
    let agents = fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
    assert!(agents.contains("DDD methodology"));
    assert!(agents.contains("Bob 4 环 Clean Architecture"));
}

#[test]
fn status_with_bob_reports_addon() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--addons", "bob"])
        .assert()
        .success();

    cli()
        .args(["status", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Addons:"))
        .stdout(predicate::str::contains("bob"))
        .stdout(predicate::str::contains("3 / 3"));
}
