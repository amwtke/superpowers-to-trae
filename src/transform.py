"""Apply mappings.json transformations to a string.

Order matters:
  1. phrase_replacements (longer multi-word patterns) — applied first
  2. tool_name_replacements (single tokens with word boundaries) — applied second

Rationale: phrases often contain tool tokens, so substituting tools first
would corrupt phrase matching.
"""
import re


def apply(text: str, mappings: dict) -> str:
    for entry in mappings.get("phrase_replacements", []):
        text = text.replace(entry["from"], entry["to"])
    for tool, replacement in mappings.get("tool_name_replacements", {}).items():
        text = re.sub(rf"\b{re.escape(tool)}\b", replacement, text)
    return text


def apply_to_file(path, mappings: dict) -> bool:
    import pathlib
    p = pathlib.Path(path)
    original = p.read_text(encoding="utf-8")
    transformed = apply(original, mappings)
    if transformed != original:
        p.write_text(transformed, encoding="utf-8")
        return True
    return False
