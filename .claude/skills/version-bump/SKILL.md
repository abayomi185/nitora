---
name: version-bump
description: Checklist-driven workflow for bumping the nitora project version. Use when the user asks to bump, tag, release, or cut a version — or when a version bump was missed as part of another change. Ensures Cargo.toml, Cargo.lock, git tag, and GitHub release stay in sync.
---

# Version Bump

## Version sources

| Source | Update |
|---|---|
| `Cargo.toml` `version` | Edit |
| `Cargo.lock` | `cargo generate-lockfile` |
| `nix/package.nix` | Automatic (`lib.importTOML ../Cargo.toml`) |
| Git tag `vX.Y.Z` | `git tag -a vX.Y.Z` |
| GitHub release | `gh release create vX.Y.Z` |

## Steps

1. Bump `version` in `Cargo.toml`, run `cargo generate-lockfile`.
2. Commit: `bump version to X.Y.Z`.
3. Tag, push, `gh release create`. Note breaking changes.
