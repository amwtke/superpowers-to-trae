//! `init` subcommand implementation.

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use include_dir::DirEntry;
use crate::{addons, agents_md, embed, rollback};

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

pub fn run(dir_arg: &str, force: bool, addons_list: &[String]) -> Result<()> {
    addons::handle_addons(addons_list)?;
    let dir = PathBuf::from(dir_arg).canonicalize().or_else(|_| {
        // If path doesn't exist yet, fall back to as-is (preflight will catch it).
        Ok::<PathBuf, anyhow::Error>(PathBuf::from(dir_arg))
    })?;
    preflight(&dir, force)?;
    run_with_options(&dir, force, /* backup_existing */ force, addons_list)
}

pub fn run_with_options(
    dir: &Path,
    _force: bool,
    backup_existing: bool,
    _addons_list: &[String],
) -> Result<()> {
    let mut session = rollback::InstallSession::new();

    // Phase 1: install rules/project_rules.md (renamed from user_rules.md in dist)
    let rules_src = embed::DIST_USER
        .get_file("rules/user_rules.md")
        .ok_or_else(|| anyhow!("embedded dist missing rules/user_rules.md"))?;
    let project_rules_body = std::str::from_utf8(rules_src.contents())?;
    let rules_target = dir.join(".trae/rules/project_rules.md");
    if let Err(e) = session.write(&rules_target, rules_src.contents(), backup_existing) {
        session.rollback();
        return Err(e);
    }

    // Phase 2: install skills/ directory recursively
    let skills_dir = embed::DIST_USER
        .get_dir("skills/superpowers")
        .ok_or_else(|| anyhow!("embedded dist missing skills/superpowers"))?;
    if let Err(e) = write_embedded_dir(skills_dir, &dir.join(".trae"), backup_existing, &mut session) {
        session.rollback();
        return Err(e);
    }

    // Phase 3: AGENTS.md (dynamically rendered)
    let skills = collect_skill_frontmatters()?;
    let agents_md_body = agents_md::render_agents_md(project_rules_body, &skills);
    let agents_target = dir.join("AGENTS.md");
    if let Err(e) = session.write(&agents_target, agents_md_body.as_bytes(), backup_existing) {
        session.rollback();
        return Err(e);
    }

    // Phase 4: install log
    let log_target = dir.join(".trae/.superpowers-install.log");
    let log_body = render_install_log(session.commit_actions_view());
    let _ = std::fs::create_dir_all(log_target.parent().unwrap());
    let _ = std::fs::write(&log_target, log_body);

    print_success(dir, skills.len());
    Ok(())
}

fn write_embedded_dir(
    src: &include_dir::Dir,
    target_root: &Path,
    backup_existing: bool,
    session: &mut rollback::InstallSession,
) -> Result<()> {
    for entry in src.entries() {
        match entry {
            DirEntry::Dir(d) => {
                write_embedded_dir(d, target_root, backup_existing, session)?;
            }
            DirEntry::File(f) => {
                let target = target_root.join(f.path());
                session.write(&target, f.contents(), backup_existing)?;
            }
        }
    }
    Ok(())
}

fn collect_skill_frontmatters() -> Result<Vec<agents_md::Frontmatter>> {
    let mut out = Vec::new();
    let dir = embed::DIST_USER
        .get_dir("skills/superpowers")
        .ok_or_else(|| anyhow!("embedded dist missing skills/superpowers"))?;
    for entry in dir.entries() {
        if let DirEntry::Dir(skill) = entry {
            // Skip `references/` — not a skill
            if skill.path().file_name().map(|n| n == "references").unwrap_or(false) {
                continue;
            }
            if let Some(skill_md) = skill.get_file(skill.path().join("SKILL.md")) {
                let text = std::str::from_utf8(skill_md.contents())?;
                let fm = agents_md::parse_skill_frontmatter(text)?;
                out.push(fm);
            }
        }
    }
    Ok(out)
}

fn render_install_log(_actions: &[rollback::RollbackAction]) -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("# install run at epoch {}\n", now)
}

fn print_success(dir: &Path, skill_count: usize) {
    use colored::Colorize;
    println!();
    println!("{} Initialized superpowers methodology in {}", "✓".green().bold(), dir.display());
    println!("  - rules:  .trae/rules/project_rules.md");
    println!("  - skills: .trae/skills/superpowers/   ({} skills)", skill_count);
    println!("  - AGENTS: AGENTS.md  (project root, double-insurance)");
    println!();
    println!("To invoke methodology in Trae IDE, include \"superpowers\" in your prompt:");
    println!("  \"Use superpowers to brainstorm <X>\"");
    println!("  \"Use superpowers to plan <Y>\"");
    println!("  \"Use superpowers to debug <Z>\"");
    println!();
    println!("Optional: for @-mention shortcuts (@brainstorm / @write-plan / etc.), see");
    println!("docs/superpowers/trae-agents-setup.md in the superpowers-trae repo for");
    println!("paste-ready Custom Agent prompts.");
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
