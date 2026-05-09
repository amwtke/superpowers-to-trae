//! Bob plugin: 3 skills + BOB.md (install-once) + README-RUN-BOB.md.

use anyhow::Result;
use include_dir::{include_dir, Dir};
use std::path::Path;
use crate::addons::Addon;
use crate::rollback::InstallSession;

static BOB_SKILLS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/bob/skills");
const ROOT_BOB_MD: &[u8] = include_bytes!("../../templates/bob/root/BOB.md");
const ROOT_README: &[u8] = include_bytes!("../../templates/bob/root/README-RUN-BOB.md");
const AGENTS_MD_SEGMENT: &str = include_str!("../../templates/bob/root/claude-md-merge.md");

pub struct BobAddon;

impl Addon for BobAddon {
    fn name(&self) -> &str {
        "bob"
    }

    fn install(
        &self,
        dir: &Path,
        backup_existing: bool,
        session: &mut InstallSession,
    ) -> Result<()> {
        // 1. Skills: each <stem>.md → .trae/skills/bob/<stem>/SKILL.md
        for entry in BOB_SKILLS.files() {
            let stem = entry
                .path()
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow::anyhow!("invalid skill filename: {:?}", entry.path()))?;
            let target = dir.join(".trae/skills/bob").join(stem).join("SKILL.md");
            session.write(&target, entry.contents(), backup_existing)?;
        }

        // 2. README-RUN-BOB.md (project root, always replace + backup)
        let readme_target = dir.join("README-RUN-BOB.md");
        session.write(&readme_target, ROOT_README, backup_existing)?;

        // 3. BOB.md (install-once: skip if exists; otherwise write without backup)
        let bob_target = dir.join("BOB.md");
        if !bob_target.exists() {
            session.write(&bob_target, ROOT_BOB_MD, false)?;
        } else {
            println!("ℹ BOB.md preserved (user-managed).");
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
    fn bob_addon_name() {
        assert_eq!(BobAddon.name(), "bob");
    }

    #[test]
    fn bob_agents_md_segment_present() {
        let segment = BobAddon.agents_md_segment().unwrap();
        assert!(segment.contains("Bob 4 环 Clean Architecture"));
        assert!(segment.contains("框架边界外推"));
    }

    #[test]
    fn bob_install_creates_skills_and_root_files() {
        use tempfile::tempdir;
        let tmp = tempdir().unwrap();
        let mut session = InstallSession::new();
        BobAddon.install(tmp.path(), false, &mut session).unwrap();

        assert!(tmp.path().join(".trae/skills/bob/bob-identify/SKILL.md").exists());
        assert!(tmp.path().join(".trae/skills/bob/bob-onion/SKILL.md").exists());
        assert!(tmp.path().join(".trae/skills/bob/bob-spec/SKILL.md").exists());
        assert!(tmp.path().join("BOB.md").exists());
        assert!(tmp.path().join("README-RUN-BOB.md").exists());
    }

    #[test]
    fn bob_install_preserves_existing_bob_md() {
        use tempfile::tempdir;
        use std::fs;
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join("BOB.md"), "USER ARCH CONTENT").unwrap();
        let mut session = InstallSession::new();
        BobAddon.install(tmp.path(), false, &mut session).unwrap();
        assert_eq!(
            fs::read_to_string(tmp.path().join("BOB.md")).unwrap(),
            "USER ARCH CONTENT"
        );
    }
}
