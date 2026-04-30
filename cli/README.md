# superpowers-trae

One-shot installer for the [Superpowers](https://github.com/obra/superpowers) methodology in [Trae IDE](https://www.trae.ai/) projects.

## Install

```bash
cargo install --git https://github.com/amwtke/superpowers-to-trae \
  --tag v0.2.2 superpowers-trae
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

## DDD plugin (v0.2+)

Add the DDD methodology harness via `--addons ddd`:

````bash
superpowers-trae init --addons ddd        # first-time init with DDD
superpowers-trae upgrade --addons ddd     # add DDD to an already-initialized project
superpowers-trae status                   # shows "Addons: ddd ✓" when installed
````

`DOMAIN.md` is install-once: once you start filling it with your domain model, subsequent `upgrade --addons ddd` runs will preserve your content (template never overwrites).

## Invocation in Trae

Tell the Builder agent:

> Use superpowers to brainstorm <X>

Optional: create 4 Custom Agents (@brainstorm / @write-plan / @execute-plan / @code-reviewer) for `@`-mention invocation. See `docs/superpowers/trae-agents-setup.md` in the parent repo.

## License

MIT
