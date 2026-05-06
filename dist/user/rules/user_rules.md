---
description: Superpowers methodology - skill discovery and use protocol
---

# You have superpowers.

You have access to a library of methodology skills under `.trae/skills/superpowers/`.
Each skill is a folder containing a SKILL.md describing when and how to use it.

## The Rule

When the user's request matches a skill's `description`, you MUST read that
skill's SKILL.md and follow it before acting — including for clarifying
questions, exploration, and planning.

If no skill matches, proceed with your normal capabilities.

## Skill Index

- brainstorming — You MUST use this before any creative work - creating features, building components, adding functionality, or modifying behavior. Explores user intent, requirements and design before implementation.
- dispatching-parallel-agents — Use when facing 2+ independent tasks that can be worked on without shared state or sequential dependencies
- executing-plans — Use when you have a written implementation plan to execute in a separate session with review checkpoints
- finishing-a-development-branch — Use when implementation is complete, all tests pass, and you need to decide how to integrate the work - guides completion of development work by presenting structured options for merge, PR, or cleanup
- receiving-code-review — Use when receiving code review feedback, before implementing suggestions, especially if feedback seems unclear or technically questionable - requires technical rigor and verification, not performative agreement or blind implementation
- requesting-code-review — Use when completing tasks, implementing major features, or before merging to verify work meets requirements
- subagent-driven-development — Use when executing implementation plans with independent tasks in the current session
- systematic-debugging — Use when encountering any bug, test failure, or unexpected behavior, before proposing fixes
- test-driven-development — Use when implementing any feature or bugfix, before writing implementation code
- using-git-worktrees — Use when starting feature work that needs isolation from current workspace or before executing implementation plans - ensures an isolated workspace exists via native tools or git worktree fallback
- using-superpowers — Use when starting any conversation - establishes how to find and use skills, requiring Skill tool invocation before ANY response including clarifying questions
- verification-before-completion — Use when about to claim work is complete, fixed, or passing, before committing or creating PRs - requires running verification commands and confirming output before making any success claims; evidence before assertions always
- writing-plans — Use when you have a spec or requirements for a multi-step task, before touching code
- writing-skills — Use when creating new skills, editing existing skills, or verifying skills work before deployment

## Tool name reference

Trae IDE tool names may differ from those mentioned in skill text. Before
following any skill that names a tool, see
`.trae/skills/superpowers/references/trae-tools.md` for mappings.

## How to load a skill

Use the Read tool on `.trae/skills/superpowers/<skill-name>/SKILL.md` and follow it
exactly. Skills evolve — do not paraphrase from memory.
