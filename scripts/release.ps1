# insomniAPP Release Script (PowerShell)
# Usage: .\scripts\release.ps1 [version] [patch|minor|major]
# Examples:
#   .\scripts\release.ps1              (auto-increment patch: 0.1.0 -> 0.1.1)
#   .\scripts\release.ps1 minor        (auto-increment minor: 0.1.0 -> 0.2.0)
#   .\scripts\release.ps1 major        (auto-increment major: 0.1.0 -> 1.0.0)
#   .\scripts\release.ps1 0.5.0        (explicit version)

param(
    [Parameter(Position=0)]
    [string]$Arg = "patch"
)

$ErrorActionPreference = "Stop"

$Root = git rev-parse --show-toplevel 2>$null
if (-not $Root) {
    Write-Host "Error: Not in a git repository" -ForegroundColor Red
    exit 1
}

# Read current version
$TauriConf = Get-Content "$Root/frontend/desktop/src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
$CurrentVersion = $TauriConf.version
$Parts = $CurrentVersion.Split('.')
$Major = [int]$Parts[0]
$Minor = [int]$Parts[1]
$Patch = [int]$Parts[2]

# Determine new version
if ($Arg -match '^\d+\.\d+\.\d+$') {
    $Version = $Arg
} elseif ($Arg -eq "patch") {
    $Version = "$Major.$Minor.$($Patch + 1)"
} elseif ($Arg -eq "minor") {
    $Version = "$Major.$($Minor + 1).0"
} elseif ($Arg -eq "major") {
    $Version = "$($Major + 1).0.0"
} else {
    Write-Host "Usage: .\scripts\release.ps1 [version|patch|minor|major]" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  patch   (default)  $CurrentVersion -> $Major.$Minor.$($Patch + 1)"
    Write-Host "  minor              $CurrentVersion -> $Major.$($Minor + 1).0"
    Write-Host "  major              $CurrentVersion -> $($Major + 1).0.0"
    Write-Host "  X.Y.Z              explicit version"
    exit 1
}

$Tag = "v$Version"

Write-Host "Current version: $CurrentVersion" -ForegroundColor Cyan
Write-Host "New version:     $Version  (tag: $Tag)" -ForegroundColor Green
Write-Host ""

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

# Confirm
$Confirm = Read-Host "Proceed? [Y/n]"
if ($Confirm -and $Confirm -ne "y" -and $Confirm -ne "Y") {
    Write-Host "Aborted."
    exit 0
}

Write-Host ""

# Bump tauri.conf.json
$TauriConfPath = "$Root/frontend/desktop/src-tauri/tauri.conf.json"
$TauriConfContent = Get-Content $TauriConfPath -Raw
$TauriConfContent = $TauriConfContent -replace [regex]::Escape("`"version`": `"$CurrentVersion`""), "`"version`": `"$Version`""
Set-Content $TauriConfPath $TauriConfContent -NoNewline
Write-Host "[1/6] Bumped tauri.conf.json" -ForegroundColor Green

# Bump Cargo.toml (only the package version, not dependency versions)
$CargoPath = "$Root/frontend/desktop/src-tauri/Cargo.toml"
$CargoContent = Get-Content $CargoPath -Raw
$CargoContent = $CargoContent -replace "(?m)^(version\s*=\s*)`"$([regex]::Escape($CurrentVersion))`"", "`$1`"$Version`""
Set-Content $CargoPath $CargoContent -NoNewline
Write-Host "[2/6] Bumped Cargo.toml" -ForegroundColor Green

# Bump package.json
$PkgPath = "$Root/frontend/desktop/package.json"
$PkgContent = Get-Content $PkgPath -Raw
$PkgContent = $PkgContent -replace [regex]::Escape("`"version`": `"$CurrentVersion`""), "`"version`": `"$Version`""
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
