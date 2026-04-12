## Homebrew tap publishing

This repo includes `.github/workflows/publish-homebrew-tap.yml`.

It updates a tap formula whenever a GitHub release is published.

### Required secret

- `HOMEBREW_TAP_TOKEN`
  - GitHub token with write access to the tap repo

### Optional repository variable

- `HOMEBREW_TAP_REPO`
  - Format: `OWNER/REPO`
  - Default if omitted: `${OWNER}/homebrew-nitora`

### Expected tap layout

- Tap repo name: usually `homebrew-nitora`
- Formula path written by the workflow: `Formula/nitora.rb`

### How it works

On release:

1. Reads package metadata from `Cargo.toml`
2. Downloads the tagged source tarball from GitHub
3. Computes the tarball SHA256
4. Writes `Formula/nitora.rb` in the tap repo
5. Commits and pushes the formula update

### Trigger manually

You can also run the workflow manually with a tag like `v0.1.0`.
