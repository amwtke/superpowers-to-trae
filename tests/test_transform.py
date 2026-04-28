import unittest
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent.parent / "src"))
import transform


class TransformTests(unittest.TestCase):
    def test_phrase_replacements_applied_in_order(self):
        text = "This runs in Claude Code."
        mappings = {
            "tool_name_replacements": {},
            "phrase_replacements": [{"from": "Claude Code", "to": "Trae IDE"}],
        }
        out = transform.apply(text, mappings)
        self.assertEqual(out, "This runs in Trae IDE.")

    def test_tool_name_word_boundary(self):
        text = "Run TaskCreate now. TaskCreated is a different word."
        mappings = {
            "tool_name_replacements": {"TaskCreate": "TraeTask"},
            "phrase_replacements": [],
        }
        out = transform.apply(text, mappings)
        self.assertEqual(out, "Run TraeTask now. TaskCreated is a different word.")

    def test_phrase_replaced_before_tool_to_avoid_double_substitution(self):
        text = "use the Skill tool to invoke X"
        mappings = {
            "tool_name_replacements": {"Skill": "TraeSkill"},
            "phrase_replacements": [{"from": "use the Skill tool to invoke", "to": "read the SKILL.md directly for"}],
        }
        out = transform.apply(text, mappings)
        self.assertEqual(out, "read the SKILL.md directly for X")

    def test_no_change_when_no_match(self):
        text = "nothing matches here"
        mappings = {"tool_name_replacements": {"Foo": "Bar"}, "phrase_replacements": []}
        self.assertEqual(transform.apply(text, mappings), text)

    def test_tool_replacement_with_backref_like_string_does_not_crash(self):
        # Regression: ensure replacement strings containing \1 or \g<x> are
        # treated as literal text, not as regex backreferences.
        text = "Use TaskCreate then exit."
        mappings = {
            "tool_name_replacements": {"TaskCreate": r"\1foo\g<bar>"},
            "phrase_replacements": [],
        }
        out = transform.apply(text, mappings)
        self.assertEqual(out, r"Use \1foo\g<bar> then exit.")


if __name__ == "__main__":
    unittest.main()
