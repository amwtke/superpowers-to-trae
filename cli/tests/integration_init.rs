//! End-to-end tests for `superpowers-trae init`.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn cli() -> Command {
    Command::cargo_bin("superpowers-trae").unwrap()
}

#[test]
fn init_writes_expected_files() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    assert!(tmp.path().join(".trae/rules/project_rules.md").exists());
    assert!(tmp.path().join(".trae/skills/superpowers/brainstorming/SKILL.md").exists());
    assert!(tmp.path().join("AGENTS.md").exists());
    assert!(tmp.path().join(".trae/.superpowers-install.log").exists());
}

#[test]
fn init_refuses_already_initialized_without_force() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already initialized"));
}

#[test]
fn init_force_creates_backup() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--force"])
        .assert()
        .success();

    let rules_dir = tmp.path().join(".trae/rules");
    let entries: Vec<_> = fs::read_dir(&rules_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(entries.iter().any(|n| n.starts_with("project_rules.md.bak.")));
}

#[test]
fn init_addons_ddd_prints_stub_and_exits_zero() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--addons", "ddd"])
        .assert()
        .success()
        .stdout(predicate::str::contains("DDD plugin not yet implemented"));
}

#[test]
fn init_addons_unknown_errors() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--addons", "bogus"])
        .assert()
        .failure();
}

#[test]
fn init_end_output_mentions_superpowers_trigger_word() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Use superpowers to"));
}

#[test]
fn agents_md_contains_directive_and_skill_index() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    let body = fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
    assert!(body.contains("AGENTS DIRECTIVE"));
    assert!(body.contains("- brainstorming —"));
    assert!(body.contains("- writing-plans —"));
}
