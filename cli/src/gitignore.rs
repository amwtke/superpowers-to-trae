//! Maintain a .gitignore block listing files managed by superpowers-trae.
//!
//! The block is delimited by markers so we can locate and replace it on every
//! init/upgrade without disturbing user-authored entries elsewhere in the file.

use anyhow::Result;
use std::path::Path;

const MARKER_BEGIN: &str = "# >>> superpowers-trae managed (do not edit) >>>";
const MARKER_END: &str = "# <<< superpowers-trae managed <<<";

pub fn ensure_gitignore(dir: &Path) -> Result<()> {
    let path = dir.join(".gitignore");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();

    let mut entries = vec![".trae/".to_string()];
    if dir.join("README-DDD-HARNESS.md").exists() {
        entries.push("README-DDD-HARNESS.md".to_string());
    }
    let body = entries.join("\n");
    let block = format!("{MARKER_BEGIN}\n{body}\n{MARKER_END}\n");

    let new_content = match (existing.find(MARKER_BEGIN), existing.find(MARKER_END)) {
        (Some(begin), Some(end)) if end > begin => {
            let after_end = end + MARKER_END.len();
            let after_end = if existing[after_end..].starts_with('\n') {
                after_end + 1
            } else {
                after_end
            };
            format!("{}{}{}", &existing[..begin], block, &existing[after_end..])
        }
        _ if existing.is_empty() => block,
        _ => {
            let sep = if existing.ends_with('\n') { "" } else { "\n" };
            format!("{existing}{sep}{block}")
        }
    };

    std::fs::write(&path, new_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn creates_gitignore_with_marker_block_when_missing() {
        let tmp = TempDir::new().unwrap();
        ensure_gitignore(tmp.path()).unwrap();
        let content = fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
        assert!(content.contains(MARKER_BEGIN));
        assert!(content.contains(".trae/"));
        assert!(content.contains(MARKER_END));
    }

    #[test]
    fn includes_readme_ddd_harness_when_file_exists() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("README-DDD-HARNESS.md"), "x").unwrap();

        ensure_gitignore(tmp.path()).unwrap();

        let content = fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
        assert!(content.contains(".trae/"));
        assert!(
            content.contains("README-DDD-HARNESS.md"),
            "DDD readme not added to ignore block"
        );
    }

    #[test]
    fn omits_readme_ddd_harness_when_file_absent() {
        let tmp = TempDir::new().unwrap();
        ensure_gitignore(tmp.path()).unwrap();
        let content = fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
        assert!(content.contains(".trae/"));
        assert!(
            !content.contains("README-DDD-HARNESS.md"),
            "DDD readme added when DDD not installed"
        );
    }

    #[test]
    fn replaces_existing_marker_block_in_place() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".gitignore");
        let initial = format!(
            "node_modules/\n{MARKER_BEGIN}\n.old-stuff/\n{MARKER_END}\n*.log\n"
        );
        std::fs::write(&path, &initial).unwrap();

        ensure_gitignore(tmp.path()).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("node_modules/"), "pre-marker user content lost");
        assert!(content.contains("*.log"), "post-marker user content lost");
        assert!(!content.contains(".old-stuff/"), "old block content not replaced");
        assert!(content.contains(".trae/"));
        assert_eq!(content.matches(MARKER_BEGIN).count(), 1, "duplicated marker");
        assert_eq!(content.matches(MARKER_END).count(), 1, "duplicated marker");
    }

    #[test]
    fn is_idempotent_when_run_twice() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join(".gitignore"), "existing/\n").unwrap();

        ensure_gitignore(tmp.path()).unwrap();
        let after_first = fs::read_to_string(tmp.path().join(".gitignore")).unwrap();

        ensure_gitignore(tmp.path()).unwrap();
        let after_second = fs::read_to_string(tmp.path().join(".gitignore")).unwrap();

        assert_eq!(after_first, after_second, "not idempotent");
    }

    #[test]
    fn appends_marker_block_when_gitignore_exists_without_markers() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".gitignore");
        std::fs::write(&path, "node_modules/\ntarget/\n").unwrap();

        ensure_gitignore(tmp.path()).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("node_modules/"), "user content lost");
        assert!(content.contains("target/"), "user content lost");
        assert!(content.contains(MARKER_BEGIN));
        assert!(content.contains(".trae/"));
        assert!(content.contains(MARKER_END));
    }
}
