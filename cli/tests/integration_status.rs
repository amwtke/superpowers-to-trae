//! End-to-end tests for `superpowers-trae status`.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn cli() -> Command {
    Command::cargo_bin("superpowers-trae").unwrap()
}

#[test]
fn status_reports_complete_install() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["status", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("project_rules.md"))
        .stdout(predicate::str::contains("14 / 14 expected"))
        .stdout(predicate::str::contains("AGENTS.md"));
}

#[test]
fn status_fails_when_uninstalled() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["status", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn status_fails_when_skills_partially_missing() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    // Delete one skill dir
    fs::remove_dir_all(tmp.path().join(".trae/skills/superpowers/brainstorming")).unwrap();

    cli()
        .args(["status", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stdout(predicate::str::contains("13 / 14 expected"));
}

#[test]
fn status_with_ddd_reports_addon() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--addons", "ddd"])
        .assert()
        .success();

    cli()
        .args(["status", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Addons:"))
        .stdout(predicate::str::contains("ddd"))
        .stdout(predicate::str::contains("3 / 3"));
}
