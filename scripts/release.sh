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
sed -i "s/\"version\": \"$CURRENT\"/\"version\": \"$VERSION\"/" "$ROOT/frontend/desktop/src-tauri/tauri.conf.json"
echo "[1/6] Bumped tauri.conf.json"

# Bump Cargo.toml (only the package version line at the top)
sed -i "0,/^version = \"$CURRENT\"/s//version = \"$VERSION\"/" "$ROOT/frontend/desktop/src-tauri/Cargo.toml"
echo "[2/6] Bumped Cargo.toml"

# Bump package.json
sed -i "s/\"version\": \"$CURRENT\"/\"version\": \"$VERSION\"/" "$ROOT/frontend/desktop/package.json"
echo "[3/6] Bumped package.json"

# Stage and commit
git add \
    "$ROOT/frontend/desktop/src-tauri/tauri.conf.json" \
    "$ROOT/frontend/desktop/src-tauri/Cargo.toml" \
    "$ROOT/frontend/desktop/package.json"
git commit -m "chore: bump version to $VERSION"
echo "[4/6] Committed version bump"

# Create tag
git tag "$TAG"
echo "[5/6] Created tag $TAG"

# Push commit and tag
git push origin main --tags
echo "[6/6] Pushed to origin"

echo ""
echo "Release $TAG triggered! Monitor at:"
echo "  https://github.com/eatbas/insomniapp/actions"
