//! `upgrade` subcommand: refresh an already-initialized project to the binary's embedded version.

use crate::{addons, commands::init};
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

pub fn run(dir_arg: &str, no_backup: bool, addons_list: &[String]) -> Result<()> {
    addons::handle_addons(addons_list)?;
    let dir = PathBuf::from(dir_arg).canonicalize().or_else(|_| {
        // Non-existent path: fall back to as-is so preflight produces a clear error.
        Ok::<PathBuf, anyhow::Error>(PathBuf::from(dir_arg))
    })?;
    preflight(&dir)?;

    // upgrade reuses the init main flow with `force=true` semantics:
    //   - default backup_existing = true (= !no_backup)
    //   - skip the "already initialized" preflight (we already verified it IS initialized)
    init::run_with_options(&dir, /* force */ true, /* backup_existing */ !no_backup, addons_list)?;
    Ok(())
}

pub fn preflight(dir: &Path) -> Result<()> {
    let rules = dir.join(".trae/rules/project_rules.md");
    if !rules.exists() {
        return Err(anyhow!(
            "not initialized at {} — use `superpowers-trae init` first",
            dir.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn preflight_fails_when_not_initialized() {
        let tmp = tempdir().unwrap();
        assert!(preflight(tmp.path()).is_err());
    }

    #[test]
    fn preflight_passes_when_initialized() {
        let tmp = tempdir().unwrap();
        let rules = tmp.path().join(".trae/rules");
        fs::create_dir_all(&rules).unwrap();
        fs::write(rules.join("project_rules.md"), "x").unwrap();
        assert!(preflight(tmp.path()).is_ok());
    }
}
