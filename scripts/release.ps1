# insomniAPP Release Script (PowerShell)
# Usage: .\scripts\release.ps1 0.2.0

param(
    [Parameter(Mandatory=$true, Position=0)]
    [string]$Version
)

$ErrorActionPreference = "Stop"

# Validate semver format
if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    Write-Host "Error: Version must be in format X.Y.Z (e.g., 0.2.0)" -ForegroundColor Red
    exit 1
}

$Tag = "v$Version"
$Root = git rev-parse --show-toplevel 2>$null
if (-not $Root) {
    Write-Host "Error: Not in a git repository" -ForegroundColor Red
    exit 1
}

# Check for uncommitted changes
$Status = git status --porcelain
if ($Status) {
    Write-Host "Error: You have uncommitted changes. Commit or stash them first." -ForegroundColor Red
    git status --short
    exit 1
}

# Check if tag already exists
$ExistingTag = git tag -l $Tag
if ($ExistingTag) {
    Write-Host "Error: Tag $Tag already exists" -ForegroundColor Red
    exit 1
}

# Read current version
$TauriConf = Get-Content "$Root/frontend/desktop/src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
$CurrentVersion = $TauriConf.version
Write-Host "Current version: $CurrentVersion" -ForegroundColor Cyan
Write-Host "New version:     $Version" -ForegroundColor Green
Write-Host ""

# Bump tauri.conf.json
$TauriConfPath = "$Root/frontend/desktop/src-tauri/tauri.conf.json"
$TauriConfContent = Get-Content $TauriConfPath -Raw
$TauriConfContent = $TauriConfContent -replace "`"version`": `"$CurrentVersion`"", "`"version`": `"$Version`""
Set-Content $TauriConfPath $TauriConfContent -NoNewline
Write-Host "[1/6] Bumped tauri.conf.json" -ForegroundColor Green

# Bump Cargo.toml (only the package version, not dependency versions)
$CargoPath = "$Root/frontend/desktop/src-tauri/Cargo.toml"
$CargoContent = Get-Content $CargoPath -Raw
$CargoContent = $CargoContent -replace "(?m)^(version\s*=\s*)`"$CurrentVersion`"", "`$1`"$Version`""
Set-Content $CargoPath $CargoContent -NoNewline
Write-Host "[2/6] Bumped Cargo.toml" -ForegroundColor Green

# Bump package.json
$PkgPath = "$Root/frontend/desktop/package.json"
$PkgContent = Get-Content $PkgPath -Raw
$PkgContent = $PkgContent -replace "`"version`": `"$CurrentVersion`"", "`"version`": `"$Version`""
Set-Content $PkgPath $PkgContent -NoNewline
Write-Host "[3/6] Bumped package.json" -ForegroundColor Green

# Stage and commit
git add $TauriConfPath $CargoPath $PkgPath
git commit -m "chore: bump version to $Version"
Write-Host "[4/6] Committed version bump" -ForegroundColor Green

# Create tag
git tag $Tag
Write-Host "[5/6] Created tag $Tag" -ForegroundColor Green

# Push commit and tag
git push origin main --tags
Write-Host "[6/6] Pushed to origin" -ForegroundColor Green

Write-Host ""
Write-Host "Release $Tag triggered! Monitor at:" -ForegroundColor Cyan
Write-Host "  https://github.com/eatbas/insomniapp/actions" -ForegroundColor Yellow
