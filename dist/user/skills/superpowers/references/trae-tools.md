# Trae IDE tool name mapping

Skills in this library were authored against Claude Code tool names. Use this
table when a skill mentions a tool name.

| Skill text mentions | Trae IDE equivalent | Notes |
|---|---|---|
| Read | Read | Same name (verify on install) |
| Edit | Edit | Same name (verify on install) |
| Write | Write | Same name (verify on install) |
| Bash | Bash | Same name (verify on install) |
| TaskCreate / TaskUpdate / TaskList | (TBD — see project README) | Trae's todo/task mechanism |
| WebSearch | (TBD) | If unavailable, do search manually |
| WebFetch | (TBD) | If unavailable, ask user to provide content |
| Skill (tool invocation) | Read the SKILL.md file directly | Trae has no "Skill tool"; load via Read |

If a skill references a tool not available in Trae, do the equivalent manually
(e.g., maintain a markdown todo list when there is no TaskCreate).

> **Note for maintainers (not consumed by the runtime agent):** The TBD entries
> are filled by the porting maintainer after verifying against a running Trae
> IDE. See `src/mappings.json` `tool_name_replacements` in the porting repo.
