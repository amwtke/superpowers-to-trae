//! `status` subcommand: report installation completeness.

use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

const EXPECTED_SKILL_COUNT: usize = 14;

pub fn run(dir_arg: &str) -> Result<()> {
    let dir = PathBuf::from(dir_arg).canonicalize()?;

    let version = env!("CARGO_PKG_VERSION");
    let upstream_version = "5.0.7";
    println!(
        "superpowers-trae v{} (embedded superpowers {}, addon plugins v{})",
        version, upstream_version, version
    );
    println!("Project: {}", dir.display());
    println!("─────────────────────────────────────────────────────────");

    let mut all_ok = true;

    let rules = dir.join(".trae/rules/project_rules.md");
    if rules.exists() {
        let size = fs::metadata(&rules).map(|m| m.len()).unwrap_or(0);
        println!("{} project_rules.md         (.trae/rules/, {} bytes)", "✓".green().bold(), size);
    } else {
        println!("{} project_rules.md         MISSING", "✗".red().bold());
        all_ok = false;
    }

    let skills_root = dir.join(".trae/skills/superpowers");
    let skill_count = count_skill_dirs(&skills_root);
    if skill_count == EXPECTED_SKILL_COUNT {
        println!("{} superpowers skills       {} / {} expected", "✓".green().bold(), skill_count, EXPECTED_SKILL_COUNT);
    } else {
        println!("{} superpowers skills       {} / {} expected", "✗".red().bold(), skill_count, EXPECTED_SKILL_COUNT);
        all_ok = false;
    }

    let agents_md = dir.join("AGENTS.md");
    if agents_md.exists() {
        let size = fs::metadata(&agents_md).map(|m| m.len()).unwrap_or(0);
        println!("{} AGENTS.md                (project root, {} bytes)", "✓".green().bold(), size);
    } else {
        println!("{} AGENTS.md                MISSING", "✗".red().bold());
        all_ok = false;
    }

    let log = dir.join(".trae/.superpowers-install.log");
    if log.exists() {
        if let Ok(meta) = fs::metadata(&log) {
            if let Ok(modified) = meta.modified() {
                let secs = modified.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                println!("{} Last init/upgrade        epoch {} (from install log mtime)", "ℹ".blue().bold(), secs);
            }
        }
    }

    println!();

    // Addons section
    check_addons(&dir, &mut all_ok);

    if !all_ok {
        anyhow::bail!("status: one or more required files missing");
    }
    Ok(())
}

fn check_addons(dir: &Path, all_ok: &mut bool) {
    let ddd_skills_root = dir.join(".trae/skills/ddd");
    let domain_md = dir.join("DOMAIN.md");
    let ddd_readme = dir.join("README-DDD-HARNESS.md");
    let has_any_ddd = ddd_skills_root.is_dir() || domain_md.exists() || ddd_readme.exists();

    let bob_skills_root = dir.join(".trae/skills/bob");
    let bob_md = dir.join("BOB.md");
    let bob_readme = dir.join("README-RUN-BOB.md");
    let has_any_bob = bob_skills_root.is_dir() || bob_md.exists() || bob_readme.exists();

    if !has_any_ddd && !has_any_bob {
        println!("Addons: none");
        return;
    }

    println!("Addons:");

    if has_any_ddd {
        let ddd_skill_count = count_skill_dirs(&ddd_skills_root);
        let ddd_complete = ddd_skill_count == 3 && domain_md.exists() && ddd_readme.exists();

        if ddd_complete {
            println!("  {} ddd", "✓".green().bold());
        } else {
            println!("  {} ddd (incomplete)", "⚠".yellow().bold());
            *all_ok = false;
        }

        println!("    - skills:    {} / 3", ddd_skill_count);

        if domain_md.exists() {
            println!("    - DOMAIN.md  (project root, user-managed)");
        } else {
            println!("    - DOMAIN.md  MISSING (run `superpowers-trae upgrade --addons ddd` to restore template)");
        }

        if ddd_readme.exists() {
            println!("    - README-DDD-HARNESS.md (project root)");
        } else {
            println!("    - README-DDD-HARNESS.md MISSING");
        }
    }

    if has_any_bob {
        let bob_skill_count = count_skill_dirs(&bob_skills_root);
        let bob_complete = bob_skill_count == 3 && bob_md.exists() && bob_readme.exists();

        if bob_complete {
            println!("  {} bob", "✓".green().bold());
        } else {
            println!("  {} bob (incomplete)", "⚠".yellow().bold());
            *all_ok = false;
        }

        println!("    - skills:    {} / 3", bob_skill_count);

        if bob_md.exists() {
            println!("    - BOB.md  (project root, user-managed)");
        } else {
            println!("    - BOB.md  MISSING (run `superpowers-trae upgrade --addons bob` to restore template)");
        }

        if bob_readme.exists() {
            println!("    - README-RUN-BOB.md (project root)");
        } else {
            println!("    - README-RUN-BOB.md MISSING");
        }
    }
}

fn count_skill_dirs(root: &Path) -> usize {
    if !root.is_dir() {
        return 0;
    }
    fs::read_dir(root)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().is_dir()
                        && e.file_name().to_str().map(|n| n != "references").unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}
