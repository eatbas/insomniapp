#!/usr/bin/env bash
# insomniAPP Release Script (Bash)
# Usage: ./scripts/release.sh 0.2.0

set -euo pipefail

VERSION="${1:-}"

if [ -z "$VERSION" ]; then
    echo "Usage: ./scripts/release.sh <version>"
    echo "Example: ./scripts/release.sh 0.2.0"
    exit 1
fi

# Validate semver format
if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "Error: Version must be in format X.Y.Z (e.g., 0.2.0)"
    exit 1
fi

TAG="v$VERSION"
ROOT=$(git rev-parse --show-toplevel 2>/dev/null) || { echo "Error: Not in a git repository"; exit 1; }

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

# Read current version from tauri.conf.json
CURRENT=$(grep -o '"version": "[^"]*"' "$ROOT/frontend/desktop/src-tauri/tauri.conf.json" | head -1 | cut -d'"' -f4)
echo "Current version: $CURRENT"
echo "New version:     $VERSION"
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
