## Homebrew tap publishing

This repo includes two workflows that work together:

- `.github/workflows/release.yml` — builds pre-compiled binaries for both `aarch64-apple-darwin` and `x86_64-apple-darwin`, packages them as tarballs, and uploads them to a GitHub release.
- `.github/workflows/publish-homebrew-tap.yml` — generates a Homebrew formula pointing at those pre-built binaries and pushes it to the tap repo.

### How it works

On tag push (`v*`):

1. `release.yml` cross-compiles for both architectures on a single `macos-latest` runner
2. Packages each binary as `nitora-<tag>-<target>.tar.gz`
3. Creates a GitHub release with both tarballs

On release published (or manual trigger):

1. `publish-homebrew-tap.yml` downloads both tarballs from the release
2. Computes SHA256 for each
3. Reads package metadata (name, description, license) from `Cargo.toml`
4. Generates a binary formula with `on_arm`/`on_intel` blocks
5. Commits and pushes `Formula/nitora.rb` to the tap repo

### User installation

```bash
brew tap abayomi185/tap
brew install nitora
```

### Required secret

- `HOMEBREW_TAP_TOKEN`
  - Fine-grained personal access token with `contents: write` scoped to the tap repo

### Optional repository variable

- `HOMEBREW_TAP_REPO`
  - Format: `OWNER/REPO`
  - Default if omitted: `${OWNER}/homebrew-tap`

### Expected tap layout

- Tap repo name: `homebrew-tap`
- Formula path written by the workflow: `Formula/nitora.rb`

### Tarball naming convention

```
nitora-v0.4.1-aarch64-apple-darwin.tar.gz
nitora-v0.4.1-x86_64-apple-darwin.tar.gz
```

### Manual prerequisites

Before the first release:

1. Create public repo `abayomi185/homebrew-tap` on GitHub (already done)
2. Generate a fine-grained PAT with `contents: write` scoped to `homebrew-tap`
3. Add the PAT as `HOMEBREW_TAP_TOKEN` in `abayomi185/nitora` → Settings → Secrets → Actions

### Trigger manually

You can run the formula publish workflow manually with a tag like `v0.4.1`.
