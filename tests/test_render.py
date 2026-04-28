import unittest
import tempfile
import pathlib
import sys
sys.path.insert(0, str(pathlib.Path(__file__).parent.parent / "src"))
import render


SKILL_FRONTMATTER_SAMPLE = """---
name: brainstorming
description: Use when starting any creative work — turn ideas into specs through dialogue
---

# Body content here
"""


class RenderTests(unittest.TestCase):
    def test_parse_skill_frontmatter_extracts_name_and_description(self):
        with tempfile.TemporaryDirectory() as td:
            p = pathlib.Path(td) / "SKILL.md"
            p.write_text(SKILL_FRONTMATTER_SAMPLE)
            fm = render.parse_skill_frontmatter(p)
        self.assertEqual(fm["name"], "brainstorming")
        self.assertIn("turn ideas", fm["description"])

    def test_render_template_substitutes_double_braces(self):
        out = render.render_template("Hello {{ name }}!", {"name": "world"})
        self.assertEqual(out, "Hello world!")

    def test_render_template_handles_multiple_vars(self):
        tpl = "{{ a }} and {{ b }}"
        self.assertEqual(render.render_template(tpl, {"a": "x", "b": "y"}), "x and y")

    def test_build_skill_index_formats_as_markdown_list(self):
        skills = [
            {"name": "alpha", "description": "Do alpha things"},
            {"name": "beta", "description": "Do beta things"},
        ]
        out = render.build_skill_index(skills)
        self.assertIn("- alpha — Do alpha things", out)
        self.assertIn("- beta — Do beta things", out)

    def test_render_template_missing_var_raises(self):
        with self.assertRaises(KeyError):
            render.render_template("{{ missing }}", {})


if __name__ == "__main__":
    unittest.main()
