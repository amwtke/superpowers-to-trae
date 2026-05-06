//! End-to-end tests for `superpowers-trae upgrade`.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn cli() -> Command {
    Command::cargo_bin("superpowers-trae").unwrap()
}

#[test]
fn upgrade_refuses_uninitialized_project() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["upgrade", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not initialized"));
}

#[test]
fn upgrade_creates_backup_by_default() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    // Modify a file so we can detect the backup
    let target = tmp.path().join(".trae/skills/superpowers/brainstorming/SKILL.md");
    fs::write(&target, "USER MODIFIED").unwrap();

    cli()
        .args(["upgrade", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    // Backup file should exist
    let parent = target.parent().unwrap();
    let entries: Vec<_> = fs::read_dir(parent)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(entries.iter().any(|n| n.starts_with("SKILL.md.bak.")));
    // Restored content should be from embedded dist (not "USER MODIFIED")
    assert_ne!(fs::read_to_string(&target).unwrap(), "USER MODIFIED");
}

#[test]
fn upgrade_no_backup_skips_backup() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    // Pre-modify a file so we can verify upgrade actually overwrote it
    // (otherwise the test could pass for the wrong reason — embedded content
    // would equal installed content and skip would be a no-op anyway).
    let target = tmp.path().join(".trae/skills/superpowers/brainstorming/SKILL.md");
    fs::write(&target, "USER MODIFIED").unwrap();

    cli()
        .args(["upgrade", "--dir", tmp.path().to_str().unwrap(), "--no-backup"])
        .assert()
        .success();

    // (a) No .bak.* files anywhere — --no-backup suppressed backup creation
    let bak_count = walkdir_count_baks(tmp.path());
    assert_eq!(bak_count, 0, "expected zero .bak.* files with --no-backup");

    // (b) The pre-modified file has been overwritten with embedded content
    assert_ne!(
        fs::read_to_string(&target).unwrap(),
        "USER MODIFIED",
        "expected upgrade to overwrite user modification"
    );
}

#[test]
fn upgrade_replaces_managed_gitignore_block_in_place() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    // 用户在 .gitignore 别处加了自己的内容（marker 块外）
    let gi_path = tmp.path().join(".gitignore");
    let initial = fs::read_to_string(&gi_path).unwrap();
    fs::write(&gi_path, format!("{initial}custom-user-rule/\n")).unwrap();

    cli()
        .args(["upgrade", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    let after = fs::read_to_string(&gi_path).unwrap();
    assert!(after.contains("custom-user-rule/"), "user content lost on upgrade");
    assert!(after.contains(".trae/"));
    // marker 不应被重复
    assert_eq!(after.matches("# >>> superpowers-trae managed").count(), 1);
}

fn walkdir_count_baks(root: &std::path::Path) -> usize {
    let mut count = 0;
    let mut stack = vec![root.to_path_buf()];
    while let Some(p) = stack.pop() {
        if let Ok(rd) = fs::read_dir(&p) {
            for e in rd.flatten() {
                let path = e.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.contains(".bak."))
                    .unwrap_or(false)
                {
                    count += 1;
                }
            }
        }
    }
    count
}
