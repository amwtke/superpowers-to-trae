//! AGENTS.md generation: combines DIRECTIVE header, skill index, and project_rules.md body.

use anyhow::{anyhow, Result};

pub struct Frontmatter {
    pub name: String,
    pub description: String,
}

/// Parse YAML-ish frontmatter (simple `key: value` lines between `---` delimiters).
/// Strips surrounding double or single quotes from values.
/// Tolerates absence of trailing newline after closing `---`.
pub fn parse_skill_frontmatter(text: &str) -> Result<Frontmatter> {
    let bytes = text.as_bytes();
    if !text.starts_with("---\n") {
        return Err(anyhow!("no frontmatter delimiter at start"));
    }
    // Find the closing "---" — line at start, followed by \n or end of string.
    let body_start = 4; // after first "---\n"
    let close = (body_start..bytes.len() - 2)
        .find(|&i| {
            (i == body_start || bytes[i - 1] == b'\n')
                && bytes[i..].starts_with(b"---")
                && (i + 3 == bytes.len() || bytes[i + 3] == b'\n')
        })
        .ok_or_else(|| anyhow!("no closing frontmatter delimiter"))?;

    let header = &text[body_start..close];
    let mut name = None;
    let mut description = None;
    for line in header.lines() {
        if let Some((k, v)) = line.split_once(':') {
            let value = strip_quotes(v.trim());
            match k.trim() {
                "name" => name = Some(value.to_string()),
                "description" => description = Some(value.to_string()),
                _ => {}
            }
        }
    }
    Ok(Frontmatter {
        name: name.ok_or_else(|| anyhow!("missing 'name' in frontmatter"))?,
        description: description.ok_or_else(|| anyhow!("missing 'description' in frontmatter"))?,
    })
}

fn strip_quotes(s: &str) -> &str {
    let bytes = s.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if first == last && (first == b'"' || first == b'\'') {
            return &s[1..s.len() - 1];
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_skill_frontmatter_extracts_name_and_description() {
        let text = "---\nname: brainstorming\ndescription: Use when starting any creative work\n---\n\nbody\n";
        let fm = parse_skill_frontmatter(text).unwrap();
        assert_eq!(fm.name, "brainstorming");
        assert!(fm.description.contains("creative work"));
    }

    #[test]
    fn parse_skill_frontmatter_strips_yaml_quotes() {
        let text = "---\nname: x\ndescription: \"quoted value\"\n---\nbody\n";
        let fm = parse_skill_frontmatter(text).unwrap();
        assert_eq!(fm.description, "quoted value");
    }

    #[test]
    fn parse_skill_frontmatter_handles_no_trailing_newline() {
        let text = "---\nname: tail\ndescription: no newline\n---";
        let fm = parse_skill_frontmatter(text).unwrap();
        assert_eq!(fm.name, "tail");
        assert_eq!(fm.description, "no newline");
    }

    #[test]
    fn parse_skill_frontmatter_missing_returns_err() {
        let text = "no frontmatter here\nfoo: bar\n";
        assert!(parse_skill_frontmatter(text).is_err());
    }
}
