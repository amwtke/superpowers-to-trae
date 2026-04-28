# superpowers-trae

One-shot installer for the [Superpowers](https://github.com/obra/superpowers) methodology in [Trae IDE](https://www.trae.ai/) projects.

## Install

```bash
cargo install --git https://github.com/amwtke/superpowers-to-trae \
  --tag v0.1.0 superpowers-trae
```

Or download a pre-built binary from [Releases](https://github.com/amwtke/superpowers-to-trae/releases).

## Usage

```bash
superpowers-trae init                     # first-time install in current dir
superpowers-trae init --dir /path/to/proj # in another dir
superpowers-trae init --force             # overwrite existing
superpowers-trae upgrade                  # refresh to embedded version (with backup)
superpowers-trae upgrade --no-backup      # skip backup
superpowers-trae status                   # check install state
```

## Invocation in Trae

Tell the Builder agent:

> Use superpowers to brainstorm <X>

Optional: create 4 Custom Agents (@brainstorm / @write-plan / @execute-plan / @code-reviewer) for `@`-mention invocation. See `docs/superpowers/trae-agents-setup.md` in the parent repo.

## License

MIT
