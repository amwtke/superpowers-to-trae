"""Template rendering and skill-frontmatter parsing for the porting build."""
import re
import pathlib


_FRONTMATTER_RE = re.compile(r"^---\s*\n(.*?)\n---\s*(?:\n|$)", re.DOTALL)
_VAR_RE = re.compile(r"\{\{\s*(\w+)\s*\}\}")


def parse_skill_frontmatter(path: "str | pathlib.Path") -> dict:
    """Read a SKILL.md and return its YAML-ish frontmatter as a dict.

    Only supports `key: value` lines (one per line). Sufficient for skills
    in this library — we don't need full YAML parsing.
    """
    text = pathlib.Path(path).read_text(encoding="utf-8")
    m = _FRONTMATTER_RE.match(text)
    if not m:
        raise ValueError(f"No frontmatter in {path}")
    out = {}
    for line in m.group(1).splitlines():
        if ":" in line:
            k, v = line.split(":", 1)
            out[k.strip()] = v.strip()
    return out


def render_template(template: str, context: dict) -> str:
    """Substitute `{{ name }}` placeholders. Raises KeyError on missing keys."""
    def sub(match):
        key = match.group(1)
        if key not in context:
            raise KeyError(f"Template variable not provided: {key}")
        return str(context[key])
    return _VAR_RE.sub(sub, template)


def build_skill_index(skills: list) -> str:
    """Render a list of {'name', 'description'} dicts as a markdown bullet list."""
    return "\n".join(f"- {s['name']} — {s['description']}" for s in skills)
