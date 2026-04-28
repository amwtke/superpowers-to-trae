"""Build helpers: orchestrate transform.py + render.py over upstream/.

This module is intentionally Python (not bash) because:
- It needs to walk directory trees applying per-file transforms.
- It needs to parse SKILL.md frontmatter to build the bootstrap index.
- It needs to render templates with multiple variables.
"""
import json
import shutil
import sys
import pathlib

import transform
import render


def load_mappings(root: pathlib.Path) -> dict:
    return json.loads((root / "src" / "mappings.json").read_text(encoding="utf-8"))


def copy_and_transform_skills(upstream: pathlib.Path, dist_skills: pathlib.Path,
                              mappings: dict) -> list:
    """Copy upstream/skills/* to dist/skills/superpowers/*, transform every .md.

    Returns a list of {'name', 'description'} dicts for the bootstrap index.
    """
    # Defensive: when called standalone (not from main, which already cleaned dist/).
    if dist_skills.exists():
        shutil.rmtree(dist_skills)
    dist_skills.mkdir(parents=True)
    skills_meta = []
    for skill_dir in sorted((upstream / "skills").iterdir()):
        if not skill_dir.is_dir():
            continue
        target = dist_skills / skill_dir.name
        shutil.copytree(skill_dir, target)
        # Drop alternate-platform reference docs — Trae users only need trae-tools.md.
        for alt in ("copilot-tools.md", "codex-tools.md", "gemini-tools.md"):
            alt_path = target / "references" / alt
            if alt_path.exists():
                alt_path.unlink()
        for md in target.rglob("*.md"):
            transform.apply_to_file(md, mappings)
        fm = render.parse_skill_frontmatter(target / "SKILL.md")
        skills_meta.append({"name": fm["name"], "description": fm["description"]})
    return skills_meta


def write_bootstrap_rule(template_path: pathlib.Path, out_path: pathlib.Path,
                        skills_meta: list) -> None:
    template = template_path.read_text(encoding="utf-8")
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(render.render_template(
        template,
        {"skill_index": render.build_skill_index(skills_meta)},
    ), encoding="utf-8")


def write_custom_agents(template_path: pathlib.Path, out_dir: pathlib.Path,
                        agents_to_generate: list, skills_meta: list) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    template = template_path.read_text(encoding="utf-8")
    for agent in agents_to_generate:
        skill_meta = next((s for s in skills_meta if s["name"] == agent["skill"]), None)
        if skill_meta is None:
            raise ValueError(f"Agent {agent['name']} references unknown skill {agent['skill']}")
        rendered = render.render_template(template, {
            "name": agent["name"],
            "skill": agent["skill"],
            "description": skill_meta["description"],
        })
        (out_dir / f"{agent['name']}.md").write_text(rendered, encoding="utf-8")


def transform_code_reviewer(upstream: pathlib.Path, out_dir: pathlib.Path,
                            mappings: dict) -> bool:
    src = upstream / "agents" / "code-reviewer.md"
    if not src.exists():
        return False
    dst = out_dir / "code-reviewer.md"
    out_dir.mkdir(parents=True, exist_ok=True)
    shutil.copy(src, dst)
    transform.apply_to_file(dst, mappings)
    return True


def install_trae_tools_ref(src_ref: pathlib.Path, dist_skills: pathlib.Path) -> None:
    target = dist_skills / "references" / "trae-tools.md"
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy(src_ref, target)


def mirror_user_to_project(user_dir: pathlib.Path, project_dir: pathlib.Path) -> None:
    if project_dir.exists():
        shutil.rmtree(project_dir)
    shutil.copytree(user_dir, project_dir)
    user_rules = project_dir / "rules" / "user_rules.md"
    project_rules = project_dir / "rules" / "project_rules.md"
    if user_rules.exists():
        user_rules.rename(project_rules)


def main(argv: list) -> None:
    root = pathlib.Path(argv[1]) if len(argv) > 1 else pathlib.Path.cwd()
    upstream = root / "upstream"
    src = root / "src"
    dist_user = root / "dist" / "user"
    dist_project = root / "dist" / "project"

    if (root / "dist").exists():
        shutil.rmtree(root / "dist")

    mappings = load_mappings(root)

    # Phase 1: skills
    skills_meta = copy_and_transform_skills(
        upstream, dist_user / "skills" / "superpowers", mappings,
    )

    # Phase 2: trae-tools.md reference
    install_trae_tools_ref(
        src / "trae-tools-reference.md",
        dist_user / "skills" / "superpowers",
    )

    # Phase 3: bootstrap rule
    write_bootstrap_rule(
        src / "bootstrap-rule.template.md",
        dist_user / "rules" / "user_rules.md",
        skills_meta,
    )

    # Phase 4: custom agents
    write_custom_agents(
        src / "agent-wrapper.template.md",
        dist_user / "agents",
        mappings["agents_to_generate"],
        skills_meta,
    )

    # Phase 5: code-reviewer
    reviewer_written = transform_code_reviewer(upstream, dist_user / "agents", mappings)

    # Phase 6: mirror to project
    mirror_user_to_project(dist_user, dist_project)

    agent_count = len(mappings["agents_to_generate"]) + (1 if reviewer_written else 0)
    print(f"Built {len(skills_meta)} skills, {agent_count} agents")
    print(f"  -> {dist_user}")
    print(f"  -> {dist_project}")


if __name__ == "__main__":
    # Python automatically adds the script's dir to sys.path[0] when invoked
    # as `python3 src/build_helpers.py`, and build.sh sets PYTHONPATH=src/.
    # No explicit insert needed.
    main(sys.argv)
