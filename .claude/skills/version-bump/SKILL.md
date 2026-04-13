---
name: release
description: Automated workflow for bumping the nitora project version. Use when the user asks to bump, tag, release, or cut a version — or when a version bump was missed as part of another change. Runs the co-located release.sh script which handles Cargo.toml, Cargo.lock, git tag, git push, and GitHub release creation in one command.
---

# Release

## Version sources

| Source                 | Update                                     |
| ---------------------- | ------------------------------------------ |
| `Cargo.toml` `version` | Edit                                       |
| `Cargo.lock`           | `cargo generate-lockfile`                  |
| `nix/package.nix`      | Automatic (`lib.importTOML ../Cargo.toml`) |
| Git tag `vX.Y.Z`       | `git tag -a vX.Y.Z`                        |
| GitHub release         | `gh release create vX.Y.Z`                 |

## Usage

```
.claude/skills/version-bump/release.sh <patch|minor|major>
```

The script handles everything: validates preconditions (clean worktree, on `main`,
`gh auth status` works), bumps the version in `Cargo.toml`, runs `cargo check` to
regenerate `Cargo.lock`, commits, tags `vX.Y.Z`, pushes with `--follow-tags`, and
creates a GitHub release via `gh release create --generate-notes`. If anything fails
after the `Cargo.toml` edit and before publishing, it rolls back automatically.

CI (`release.yml`) builds the binary and uploads release assets for the pushed tag.
`publish-homebrew-tap.yml` updates the Homebrew formula automatically.

