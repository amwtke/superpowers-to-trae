//! `init` subcommand implementation.

use anyhow::{anyhow, Result};
use std::path::Path;

/// Pre-install validation. Returns Err if the project cannot accept install.
pub fn preflight(dir: &Path, force: bool) -> Result<()> {
    if !dir.exists() {
        return Err(anyhow!("target directory does not exist: {}", dir.display()));
    }
    if !dir.is_dir() {
        return Err(anyhow!("target is not a directory: {}", dir.display()));
    }
    let rules = dir.join(".trae/rules/project_rules.md");
    if rules.exists() && !force {
        return Err(anyhow!(
            "already initialized at {} — use `superpowers-trae upgrade` (or pass --force to reinit)",
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
    fn preflight_passes_on_empty_dir() {
        let tmp = tempdir().unwrap();
        assert!(preflight(tmp.path(), false).is_ok());
    }

    #[test]
    fn preflight_fails_on_nonexistent() {
        let r = preflight(Path::new("/nonexistent/qwerty"), false);
        assert!(r.is_err());
    }

    #[test]
    fn preflight_fails_on_already_initialized_without_force() {
        let tmp = tempdir().unwrap();
        let rules = tmp.path().join(".trae/rules");
        fs::create_dir_all(&rules).unwrap();
        fs::write(rules.join("project_rules.md"), "x").unwrap();
        let r = preflight(tmp.path(), false);
        assert!(r.is_err());
        assert!(format!("{:?}", r.unwrap_err()).contains("already initialized"));
    }

    #[test]
    fn preflight_passes_on_already_initialized_with_force() {
        let tmp = tempdir().unwrap();
        let rules = tmp.path().join(".trae/rules");
        fs::create_dir_all(&rules).unwrap();
        fs::write(rules.join("project_rules.md"), "x").unwrap();
        assert!(preflight(tmp.path(), true).is_ok());
    }
}
