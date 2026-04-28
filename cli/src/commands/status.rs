//! `status` subcommand: report installation completeness.

use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

const EXPECTED_SKILL_COUNT: usize = 14;

pub fn run(dir_arg: &str) -> Result<()> {
    let dir = PathBuf::from(dir_arg).canonicalize()?;

    let version = env!("CARGO_PKG_VERSION");
    let upstream_version = "5.0.7"; // matches Cargo.toml [package.metadata.upstream]
    println!("superpowers-trae v{} (embedded superpowers {})", version, upstream_version);
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
    println!("Addons: none");

    if !all_ok {
        anyhow::bail!("status: one or more required files missing");
    }
    Ok(())
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
