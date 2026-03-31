# Contributing

## Workflow

- Rebase-first: **always rebase** feature branches onto their base, never merge. Use `git pull --rebase`.
- `main` is stable and tagged. Only `core/dev` merges into it (via fast-forward or a clean rebase).
- Feature branches are cut from `core/dev` and rebased back onto it before merging.

```
main
  core/dev
    tui/feat/vim-motions
    lang/fix/parser-panic
    feat/unified-graph
```

## Branch naming

### Permanent branches

| Branch     | Purpose                                      |
|------------|----------------------------------------------|
| `main`     | Stable, tagged releases                      |
| `core/dev` | Integration — all feature branches land here |

### Feature branches

**Crate-specific work** — `<crate>/<type>/<short-description>`

| Crate prefix | Crate                  |
|--------------|------------------------|
| `rustic`     | rustic (core engine)   |
| `derive`     | rustic-derive          |
| `keyboard`   | rustic-keyboard        |
| `lang`       | rustic-lang            |
| `meta`       | rustic-meta            |
| `tui`        | rustic-tui             |
| `toolkit`    | rustic-toolkit         |

**Cross-crate work** — `<type>/<short-description>`

| Type      | When to use                                  |
|-----------|----------------------------------------------|
| `feat`    | New feature                                  |
| `fix`     | Bug fix                                      |
| `rework`  | Significant refactor / architectural change  |
| `chore`   | Tooling, CI, dependencies, config            |
| `docs`    | Documentation only                           |

### Examples

```
tui/feat/vim-motions          # new feature in rustic-tui
lang/fix/parser-panic         # bug fix in rustic-lang
rustic/rework/filter-graph    # refactor in core engine
feat/unified-graph            # cross-crate new feature
chore/update-pre-commit       # tooling change (no crate scope needed)
docs/architecture             # documentation
```

## Commit messages

```
[Add]  new thing
[Fix]  what was broken
[Upd]  changed/improved thing
[Del]  removed thing
[Rework] significant restructure
```

Keep the subject line short. Add a body if the why isn't obvious from the diff.
