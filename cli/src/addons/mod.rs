//! Addon plugin scaffolding (v1: stub only).
//!
//! v0.2 will wire actual implementations behind this trait.

use anyhow::{bail, Result};

pub trait Addon {
    fn name(&self) -> &str;
}

/// Validate and respond to `--addons` flag values.
/// In v0.1 only "ddd" is recognized (and is a no-op stub).
/// Unknown names error out.
pub fn handle_addons(addons: &[String]) -> Result<()> {
    for a in addons {
        match a.as_str() {
            "ddd" => {
                println!("ℹ DDD plugin not yet implemented in v1.");
                println!("  Scheduled for sub-project 2 (see roadmap).");
                println!("  Continuing with base superpowers only.");
            }
            other => {
                bail!(
                    "Unknown addon: '{}' (supported in v0.1: 'ddd' [stub])",
                    other
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_addons_ok() {
        assert!(handle_addons(&[]).is_ok());
    }

    #[test]
    fn ddd_stub_ok() {
        assert!(handle_addons(&["ddd".to_string()]).is_ok());
    }

    #[test]
    fn unknown_addon_errors() {
        let r = handle_addons(&["unknown".to_string()]);
        assert!(r.is_err());
        assert!(format!("{:?}", r.unwrap_err()).contains("Unknown addon"));
    }
}
