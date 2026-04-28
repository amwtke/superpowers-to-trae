//! Atomic-ish file installer with rollback on failure.

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum RollbackAction {
    Created(PathBuf),
    Backed { original: PathBuf, backup: PathBuf },
}

#[derive(Default)]
pub struct InstallSession {
    actions: Vec<RollbackAction>,
}

impl InstallSession {
    pub fn new() -> Self {
        Self::default()
    }

    /// Write `bytes` to `target`, optionally backing up existing content for rollback.
    ///
    /// - If `target` does not exist: writes the file and records a `Created` action.
    /// - If `target` exists and `backup_existing` is `true`: renames the existing file
    ///   to `<target>.bak.<timestamp>`, writes new content, and records a `Backed` action.
    /// - If `target` exists and `backup_existing` is `false`: **overwrites in place**;
    ///   the original content is **not recoverable via `rollback()`**.
    ///
    /// Pass `backup_existing = true` whenever rollback safety on pre-existing files
    /// matters. Pass `false` only when the caller has confirmed `target` does not exist
    /// (e.g., after a preflight check).
    pub fn write(&mut self, target: &Path, bytes: &[u8], backup_existing: bool) -> Result<()> {
        if target.exists() {
            if backup_existing {
                let bak = target.with_file_name(format!(
                    "{}.bak.{}",
                    target.file_name().unwrap().to_string_lossy(),
                    timestamp(),
                ));
                fs::rename(target, &bak)?;
                self.actions.push(RollbackAction::Backed {
                    original: target.to_path_buf(),
                    backup: bak,
                });
            } else {
                // overwrite path: no backup recorded (caller responsibility)
            }
        } else {
            self.actions.push(RollbackAction::Created(target.to_path_buf()));
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(target, bytes)?;
        Ok(())
    }

    /// Roll back all recorded actions in reverse order. Best-effort.
    pub fn rollback(self) {
        for action in self.actions.into_iter().rev() {
            match action {
                RollbackAction::Created(p) => {
                    let _ = fs::remove_file(&p);
                }
                RollbackAction::Backed { original, backup } => {
                    let _ = fs::remove_file(&original);
                    let _ = fs::rename(&backup, &original);
                }
            }
        }
    }

    /// Consume this session and return the recorded action log.
    ///
    /// Primary use is testing: callers in production code typically don't
    /// inspect the log on the success path — letting the session drop without
    /// calling `commit()` or `rollback()` is fine.
    #[must_use]
    pub fn commit(self) -> Vec<RollbackAction> {
        self.actions
    }
}

fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    // Epoch seconds — cheap uniqueness without chrono dep. Human-readable format not required.
    format!("{}", secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn write_creates_new_file_and_records_created() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("a.txt");
        let mut s = InstallSession::new();
        s.write(&path, b"hello", false).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");
        let acts = s.commit();
        assert_eq!(acts.len(), 1);
        matches!(&acts[0], RollbackAction::Created(p) if p == &path);
    }

    #[test]
    fn write_backups_existing_when_backup_existing_true() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("a.txt");
        fs::write(&path, b"old").unwrap();

        let mut s = InstallSession::new();
        s.write(&path, b"new", true).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "new");
        let acts = s.commit();
        assert_eq!(acts.len(), 1);
        // The backup file should exist
        match &acts[0] {
            RollbackAction::Backed { backup, .. } => {
                assert!(backup.exists());
                assert_eq!(fs::read_to_string(backup).unwrap(), "old");
            }
            _ => panic!("expected Backed action"),
        }
    }

    #[test]
    fn rollback_restores_state_after_partial_install() {
        let tmp = tempdir().unwrap();
        let preexisting = tmp.path().join("a.txt");
        fs::write(&preexisting, b"original_a").unwrap();

        let new_file = tmp.path().join("b.txt");

        let mut s = InstallSession::new();
        s.write(&preexisting, b"replaced_a", true).unwrap();
        s.write(&new_file, b"created_b", false).unwrap();

        s.rollback();

        assert_eq!(fs::read_to_string(&preexisting).unwrap(), "original_a");
        assert!(!new_file.exists());
    }
}
