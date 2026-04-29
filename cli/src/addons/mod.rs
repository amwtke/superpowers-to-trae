//! Addon plugin scaffolding.
//!
//! v0.2 changes vs v0.1:
//! - `Addon` trait gains `install()` + `agents_md_segment()`
//! - `resolve_addons` returns concrete addon objects (replaces stub-only `handle_addons`)
//! - `handle_addons` retained as backward-compatible alias

use anyhow::{bail, Result};
use std::path::Path;
use crate::rollback::InstallSession;

// pub mod ddd;  // re-enabled by Task 4

pub trait Addon {
    fn name(&self) -> &str;

    /// Install/refresh the plugin in the target dir.
    /// Uses the rollback session for backup safety.
    fn install(
        &self,
        dir: &Path,
        backup_existing: bool,
        session: &mut InstallSession,
    ) -> Result<()>;

    /// Optional AGENTS.md segment to append to the directive (compile-time embedded).
    /// Default: None means this addon does not extend AGENTS.md.
    fn agents_md_segment(&self) -> Option<&'static str> {
        None
    }
}

/// Validate `--addons` names and return concrete addon objects.
/// Empty input → empty vec. Unknown name → error.
pub fn resolve_addons(addons: &[String]) -> Result<Vec<Box<dyn Addon>>> {
    let mut out: Vec<Box<dyn Addon>> = Vec::new();
    for a in addons {
        match a.as_str() {
            "ddd" => bail!("ddd addon temporarily disabled (Task 4 will populate)"),
            other => bail!(
                "Unknown addon: '{}' (supported in v0.2: 'ddd')",
                other
            ),
        }
    }
    Ok(out)
}

/// Backward-compatible alias for v0.1 callers (drops the addon objects, just validates names).
/// Retains v0.1 ddd stub behavior (prints informational message, exits zero) until Task 4.
pub fn handle_addons(addons: &[String]) -> Result<()> {
    for a in addons {
        match a.as_str() {
            "ddd" => {
                println!("ℹ DDD plugin not yet implemented in v1.");
                println!("  Scheduled for sub-project 2 (see roadmap).");
                println!("  Continuing with base superpowers only.");
            }
            other => bail!(
                "Unknown addon: '{}' (supported in v0.2: 'ddd')",
                other
            ),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_empty_ok() {
        assert_eq!(resolve_addons(&[]).unwrap().len(), 0);
    }

    #[test]
    #[ignore = "re-enabled by Task 4"]
    fn resolve_ddd_returns_one_addon() {
        let addons = resolve_addons(&["ddd".to_string()]).unwrap();
        assert_eq!(addons.len(), 1);
        assert_eq!(addons[0].name(), "ddd");
    }

    #[test]
    fn resolve_unknown_errors() {
        let r = resolve_addons(&["bogus".to_string()]);
        assert!(r.is_err());
        assert!(format!("{:?}", r.err().unwrap()).contains("Unknown addon"));
    }

    #[test]
    fn handle_addons_alias_works_for_empty() {
        assert!(handle_addons(&[]).is_ok());
    }

    #[test]
    #[ignore = "re-enabled by Task 4"]
    fn handle_addons_alias_works_for_ddd() {
        assert!(handle_addons(&["ddd".to_string()]).is_ok());
    }

    #[test]
    fn handle_addons_alias_errors_on_unknown() {
        assert!(handle_addons(&["bogus".to_string()]).is_err());
    }
}
