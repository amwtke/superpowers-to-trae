//! DDD plugin: 3 skills + DOMAIN.md (install-once) + README-DDD-HARNESS.md.

use anyhow::Result;
use include_dir::{include_dir, Dir};
use std::path::Path;
use crate::addons::Addon;
use crate::rollback::InstallSession;

static DDD_SKILLS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/ddd/skills");
const ROOT_DOMAIN_MD: &[u8] = include_bytes!("../../templates/ddd/root/DOMAIN.md");
const ROOT_README: &[u8] = include_bytes!("../../templates/ddd/root/README-DDD-HARNESS.md");
const AGENTS_MD_SEGMENT: &str = include_str!("../../templates/ddd/root/claude-md-merge.md");

pub struct DddAddon;

impl Addon for DddAddon {
    fn name(&self) -> &str {
        "ddd"
    }

    fn install(
        &self,
        dir: &Path,
        backup_existing: bool,
        session: &mut InstallSession,
    ) -> Result<()> {
        // 1. Skills: each <stem>.md → .trae/skills/ddd/<stem>/SKILL.md
        for entry in DDD_SKILLS.files() {
            let stem = entry
                .path()
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow::anyhow!("invalid skill filename: {:?}", entry.path()))?;
            let target = dir.join(".trae/skills/ddd").join(stem).join("SKILL.md");
            session.write(&target, entry.contents(), backup_existing)?;
        }

        // 2. README-DDD-HARNESS.md (project root, always replace + backup)
        let readme_target = dir.join("README-DDD-HARNESS.md");
        session.write(&readme_target, ROOT_README, backup_existing)?;

        // 3. DOMAIN.md (install-once: skip if exists; otherwise write without backup)
        let domain_target = dir.join("DOMAIN.md");
        if !domain_target.exists() {
            session.write(&domain_target, ROOT_DOMAIN_MD, false)?;
        } else {
            println!("ℹ DOMAIN.md preserved (user-managed).");
        }

        Ok(())
    }

    fn agents_md_segment(&self) -> Option<&'static str> {
        Some(AGENTS_MD_SEGMENT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ddd_addon_name() {
        assert_eq!(DddAddon.name(), "ddd");
    }

    #[test]
    fn ddd_agents_md_segment_present() {
        let segment = DddAddon.agents_md_segment().unwrap();
        assert!(segment.contains("DDD methodology"));
        assert!(segment.contains("Ubiquitous Language"));
    }

    #[test]
    fn ddd_install_creates_skills_and_root_files() {
        use tempfile::tempdir;
        let tmp = tempdir().unwrap();
        let mut session = InstallSession::new();
        DddAddon.install(tmp.path(), false, &mut session).unwrap();

        assert!(tmp.path().join(".trae/skills/ddd/ddd-storm/SKILL.md").exists());
        assert!(tmp.path().join(".trae/skills/ddd/ddd-model/SKILL.md").exists());
        assert!(tmp.path().join(".trae/skills/ddd/ddd-spec/SKILL.md").exists());
        assert!(tmp.path().join("DOMAIN.md").exists());
        assert!(tmp.path().join("README-DDD-HARNESS.md").exists());
    }

    #[test]
    fn ddd_install_preserves_existing_domain_md() {
        use tempfile::tempdir;
        use std::fs;
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join("DOMAIN.md"), "USER MODEL CONTENT").unwrap();
        let mut session = InstallSession::new();
        DddAddon.install(tmp.path(), false, &mut session).unwrap();
        assert_eq!(
            fs::read_to_string(tmp.path().join("DOMAIN.md")).unwrap(),
            "USER MODEL CONTENT"
        );
    }
}
