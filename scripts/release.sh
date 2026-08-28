#!/usr/bin/env bash
# insomniAPP Release Script (Bash)
# Usage: ./scripts/release.sh [version] [patch|minor|major]
# Examples:
#   ./scripts/release.sh              (auto-increment patch: 0.1.0 -> 0.1.1)
#   ./scripts/release.sh minor        (auto-increment minor: 0.1.0 -> 0.2.0)
#   ./scripts/release.sh major        (auto-increment major: 0.1.0 -> 1.0.0)
#   ./scripts/release.sh 0.5.0        (explicit version)

set -euo pipefail

ROOT=$(git rev-parse --show-toplevel 2>/dev/null) || { echo "Error: Not in a git repository"; exit 1; }

# Replaces the first line containing `needle`, swapping `needle` for
# `replacement` on that line only.
#
# Written with awk rather than `sed -i` because both `-i` without a backup
# suffix and the `0,/re/` first-match address are GNU extensions. The BSD sed
# that ships with macOS rejects them, and this script is documented as the
# macOS release path.
replace_first() {
    local file="$1" needle="$2" replacement="$3" tmp
    tmp=$(mktemp)
    awk -v needle="$needle" -v replacement="$replacement" '
        !done {
            idx = index($0, needle)
            if (idx > 0) {
                $0 = substr($0, 1, idx - 1) replacement substr($0, idx + length(needle))
                done = 1
            }
        }
        { print }
    ' "$file" > "$tmp" && mv "$tmp" "$file"
}

# Rewrites the version of the `insomniapp` package in Cargo.lock.
#
# The version line is only meaningful in context: plenty of dependencies share
# any given version string, so this anchors on the package name and edits the
# line immediately after it.
bump_lockfile_version() {
    local file="$1" version="$2" tmp
    tmp=$(mktemp)
    awk -v version="$version" '
        /^name = "insomniapp"$/ {
            print
            if ((getline) <= 0) next
            sub(/^version = ".*"$/, "version = \"" version "\"")
        }
        { print }
    ' "$file" > "$tmp" && mv "$tmp" "$file"
}

# Read current version from tauri.conf.json
CURRENT=$(grep -o '"version": "[^"]*"' "$ROOT/frontend/desktop/src-tauri/tauri.conf.json" | head -1 | cut -d'"' -f4)
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT"

ARG="${1:-patch}"

# Determine new version
if echo "$ARG" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    VERSION="$ARG"
elif [ "$ARG" = "patch" ]; then
    VERSION="$MAJOR.$MINOR.$((PATCH + 1))"
elif [ "$ARG" = "minor" ]; then
    VERSION="$MAJOR.$((MINOR + 1)).0"
elif [ "$ARG" = "major" ]; then
    VERSION="$((MAJOR + 1)).0.0"
else
    echo "Usage: ./scripts/release.sh [version|patch|minor|major]"
    echo ""
    echo "  patch   (default)  $CURRENT -> $MAJOR.$MINOR.$((PATCH + 1))"
    echo "  minor              $CURRENT -> $MAJOR.$((MINOR + 1)).0"
    echo "  major              $CURRENT -> $((MAJOR + 1)).0.0"
    echo "  X.Y.Z              explicit version"
    exit 1
fi

TAG="v$VERSION"

echo "Current version: $CURRENT"
echo "New version:     $VERSION  (tag: $TAG)"
echo ""

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo "Error: You have uncommitted changes. Commit or stash them first."
    git status --short
    exit 1
fi

# Check if tag already exists
if git tag -l "$TAG" | grep -q "$TAG"; then
    echo "Error: Tag $TAG already exists"
    exit 1
fi

# Confirm
read -rp "Proceed? [Y/n] " CONFIRM
if [ -n "$CONFIRM" ] && [ "$CONFIRM" != "y" ] && [ "$CONFIRM" != "Y" ]; then
    echo "Aborted."
    exit 0
fi

echo ""

# Bump tauri.conf.json
replace_first "$ROOT/frontend/desktop/src-tauri/tauri.conf.json" \
    "\"version\": \"$CURRENT\"" "\"version\": \"$VERSION\""
echo "[1/7] Bumped tauri.conf.json"

# Bump Cargo.toml (only the package version line at the top)
replace_first "$ROOT/frontend/desktop/src-tauri/Cargo.toml" \
    "version = \"$CURRENT\"" "version = \"$VERSION\""
echo "[2/7] Bumped Cargo.toml"

# Bump package.json
replace_first "$ROOT/frontend/desktop/package.json" \
    "\"version\": \"$CURRENT\"" "\"version\": \"$VERSION\""
echo "[3/7] Bumped package.json"

# Bump Cargo.lock. Cargo would rewrite this on the next build anyway, but a lock
# file that disagrees with Cargo.toml makes every build dirty the working tree
# and breaks any `--locked` build outright.
bump_lockfile_version "$ROOT/frontend/desktop/src-tauri/Cargo.lock" "$VERSION"
echo "[4/7] Bumped Cargo.lock"

# Stage and commit
git add \
    "$ROOT/frontend/desktop/src-tauri/tauri.conf.json" \
    "$ROOT/frontend/desktop/src-tauri/Cargo.toml" \
    "$ROOT/frontend/desktop/src-tauri/Cargo.lock" \
    "$ROOT/frontend/desktop/package.json"
git commit -m "chore: bump version to $VERSION"
echo "[5/7] Committed version bump"

# Create tag
git tag "$TAG"
echo "[6/7] Created tag $TAG"

# Push commit and tag
git push origin main --tags
echo "[7/7] Pushed to origin"

echo ""
echo "Release $TAG triggered! Monitor at:"
echo "  https://github.com/eatbas/insomniapp/actions"
