#!/usr/bin/env bash
# Usage: ./scripts/check-release.sh [BASE_REF]
# BASE_REF defaults to origin/main (the branch a PR targets).
#
# Exits 0 if at least one component has a version bump and no tag conflicts.
# Exits 1 otherwise — used as a PR gate.
set -euo pipefail

BASE_REF="${1:-origin/main}"
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

FAIL=0
BUMPED=0

# helpers

cargo_version_current() { grep '^version' "$1" | head -1 | sed 's/version = "\(.*\)"/\1/'; }
cargo_version_base()    { git show "$BASE_REF:$1" 2>/dev/null | grep '^version' | head -1 | sed 's/version = "\(.*\)"/\1/' || true; }
py_version_current()    { grep '^version' "$1" | sed 's/version = "\(.*\)"/\1/'; }
py_version_base()       { git show "$BASE_REF:$1" 2>/dev/null | grep '^version' | sed 's/version = "\(.*\)"/\1/' || true; }

check_tag_free() {
  local tag="$1"
  if git tag --list "$tag" | grep -q .; then
    echo "      ✘ tag '$tag' already exists — bump the version again or delete the tag"
    FAIL=1
  fi
}

check_component() {
  local name="$1" manifest="$2" tag_prefix="$3" kind="$4"
  local current="" base=""

  if [[ "$kind" == "cargo" ]]; then
    current=$(cargo_version_current "$manifest" 2>/dev/null || true)
    base=$(cargo_version_base "$manifest")
  else
    current=$(py_version_current "$manifest" 2>/dev/null || true)
    base=$(py_version_base "$manifest")
  fi

  if [[ -z "$current" ]]; then
    echo "  ?  $name — could not read version from $manifest"
    return
  fi

  if [[ "$current" == "$base" ]]; then
    printf "  ·  %-20s %s\n" "$name" "$current"
  else
    printf "  ↑  %-20s %s → %s   (would tag: %s/v%s)\n" \
      "$name" "${base:-(new)}" "$current" "$tag_prefix" "$current"
    check_tag_free "$tag_prefix/v$current"
    BUMPED=$((BUMPED + 1))
  fi
}

# git state checks

echo "==> Comparing against $BASE_REF"
echo ""

if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "  ✘ Working tree has uncommitted changes — commit or stash first."
  FAIL=1
fi

# component checks

echo "Rust crates:"
check_component "rustic"          rustic/Cargo.toml          rustic          cargo
check_component "rustic-meta"     rustic-meta/Cargo.toml     rustic-meta     cargo
check_component "rustic-derive"   rustic-derive/Cargo.toml   rustic-derive   cargo
check_component "rustic-lang"     rustic-lang/Cargo.toml     rustic-lang     cargo
check_component "rustic-tui"      rustic-tui/Cargo.toml      rustic-tui      cargo
check_component "rustic-keyboard" rustic-keyboard/Cargo.toml rustic-keyboard cargo

echo ""
echo "Python packages:"
check_component "rustic-py"       rustic-py/pyproject.toml   rustic-py       python

echo ""

# summary

if [[ "$BUMPED" -eq 0 ]]; then
  echo "✘  No version bumps detected. PRs to main must include at least one version bump."
  FAIL=1
else
  echo "✔  $BUMPED component(s) ready to release."
fi

exit "$FAIL"
