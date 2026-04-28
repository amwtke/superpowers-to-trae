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

/// Render the AGENTS.md content shipped to project root.
///
/// Two parts:
///   1. AGENTS DIRECTIVE header — strong imperative tied to skill loading.
///   2. Verbatim copy of project_rules.md body (double-insurance redundancy).
pub fn render_agents_md(project_rules_body: &str, skills: &[Frontmatter]) -> String {
    let skill_index = skills
        .iter()
        .map(|s| format!("- {} — {}", s.name, s.description))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"# AGENTS DIRECTIVE — superpowers methodology

> **Critical**: When the user's request matches a skill's purpose listed below,
> you MUST `Read` the corresponding SKILL.md file and follow it exactly BEFORE
> producing any other output (including clarifying questions, exploration, or
> proposed solutions). Do not paraphrase from memory — skills evolve and the
> file is the source of truth.

## How to load a skill

1. Identify which skill matches the user's request (see Skill Index below).
2. Use the `Read` tool on `.trae/skills/superpowers/<skill-name>/SKILL.md`.
3. Follow that file's checklist verbatim.

## Skill Index

{skill_index}

## Tool name reference

For tool name mappings between Claude Code and Trae IDE, see
`.trae/skills/superpowers/references/trae-tools.md`.

---

{project_rules_body}"#,
        skill_index = skill_index,
        project_rules_body = project_rules_body,
    )
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

    #[test]
    fn render_agents_md_includes_directive_header() {
        let out = render_agents_md("rule body\n", &[]);
        assert!(out.contains("# AGENTS DIRECTIVE — superpowers methodology"));
        assert!(out.contains("you MUST `Read`"));
    }

    #[test]
    fn render_agents_md_includes_skill_index() {
        let skills = vec![
            Frontmatter { name: "alpha".into(), description: "Do alpha".into() },
            Frontmatter { name: "beta".into(), description: "Do beta".into() },
        ];
        let out = render_agents_md("rule body\n", &skills);
        assert!(out.contains("- alpha — Do alpha"));
        assert!(out.contains("- beta — Do beta"));
    }

    #[test]
    fn render_agents_md_appends_project_rules_body() {
        let out = render_agents_md("PROJECT RULES BODY\n", &[]);
        assert!(out.contains("PROJECT RULES BODY"));
        // Body should appear AFTER the DIRECTIVE section (separated by ---)
        let directive_pos = out.find("AGENTS DIRECTIVE").unwrap();
        let body_pos = out.find("PROJECT RULES BODY").unwrap();
        assert!(directive_pos < body_pos);
    }
}
