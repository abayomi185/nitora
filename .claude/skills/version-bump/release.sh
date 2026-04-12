#!/usr/bin/env bash
#
# Automate version bump, commit, tag, and push for nitora releases.
#
# Usage: .claude/skills/version-bump/release.sh <patch|minor|major>
#
# Prerequisites: clean worktree, on main branch, cargo available.
# On failure after Cargo.toml edit, rolls back automatically.

set -euo pipefail

readonly CARGO_TOML="Cargo.toml"

# --- helpers ----------------------------------------------------------------

die() { printf 'error: %s\n' "$1" >&2; exit 1; }

rollback() {
    printf 'Rolling back %s and Cargo.lock...\n' "$CARGO_TOML" >&2
    git checkout -- "$CARGO_TOML" Cargo.lock 2>/dev/null || true
}

# Increment a semver component. Resets trailing components to 0.
# Usage: bump_version <major> <minor> <patch> <part>
bump_version() {
    local major=$1 minor=$2 patch=$3 part=$4
    case $part in
        major) printf '%d.%d.%d' "$(( major + 1 ))" 0 0 ;;
        minor) printf '%d.%d.%d' "$major" "$(( minor + 1 ))" 0 ;;
        patch) printf '%d.%d.%d' "$major" "$minor" "$(( patch + 1 ))" ;;
    esac
}

# --- preconditions ----------------------------------------------------------

[[ $# -eq 1 ]] || die "usage: release.sh <patch|minor|major>"

part="$1"
case $part in
    patch|minor|major) ;;
    *) die "argument must be one of: patch, minor, major (got '$part')" ;;
esac

[[ -z "$(git status --porcelain)" ]] || die "worktree is dirty — commit or stash first"

branch="$(git rev-parse --abbrev-ref HEAD)"
[[ "$branch" == "main" ]] || die "must be on main branch (currently on '$branch')"

# --- parse current version --------------------------------------------------

# Match the first `version = "X.Y.Z"` that appears under [package].
# Cargo.toml puts [package] version early; we grab only the first match.
current="$(grep -m1 '^version = "' "$CARGO_TOML" | sed 's/version = "\(.*\)"/\1/')"
[[ -n "$current" ]] || die "could not parse version from $CARGO_TOML"

IFS='.' read -r major minor patch <<< "$current"

# Sanity: all components must be numeric.
[[ "$major$minor$patch" =~ ^[0-9]+$ ]] || die "parsed version '$current' contains non-numeric components"

# --- compute new version ----------------------------------------------------

new_version="$(bump_version "$major" "$minor" "$patch" "$part")"
tag="v${new_version}"

printf '%s → %s (%s bump)\n' "$current" "$new_version" "$part"

# --- check tag does not exist -----------------------------------------------

git fetch --tags --quiet
if git rev-parse "$tag" >/dev/null 2>&1; then
    die "tag '$tag' already exists"
fi

# --- update Cargo.toml ------------------------------------------------------

# From this point on, rollback if anything fails.
trap rollback ERR

sed -i '' "s/^version = \"${current}\"/version = \"${new_version}\"/" "$CARGO_TOML"

# Verify the edit actually landed.
grep -q "^version = \"${new_version}\"" "$CARGO_TOML" \
    || die "sed replacement did not produce expected version in $CARGO_TOML"

# --- cargo check (regenerates Cargo.lock, validates) ------------------------

printf 'Running cargo check...\n'
cargo check --quiet

# --- commit, tag, push ------------------------------------------------------

git add "$CARGO_TOML" Cargo.lock
git commit --quiet -m "chore: bump version to ${new_version}"
git tag "$tag"
git push origin main --follow-tags --quiet

trap - ERR

# --- summary ----------------------------------------------------------------

cat <<EOF

  Release prepared:
    ${current} → ${new_version}
    Tag ${tag} pushed to origin.
    CI will build the release and update the Homebrew tap.

EOF
