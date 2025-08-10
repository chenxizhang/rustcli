param(
    [ValidateSet('patch','minor','major')][string]$Bump = 'patch',
    [string]$Message = ''
)

$ErrorActionPreference = 'Stop'

function Get-RepoRoot {
    param([string]$Start)
    $d = Resolve-Path $Start
    while ($d -and -not (Test-Path (Join-Path $d '.git'))) {
        $parent = Split-Path -Parent $d
        if ($parent -eq $d) { break }
        $d = $parent
    }
    if (-not (Test-Path (Join-Path $d '.git'))) { throw 'Not inside a Git repository.' }
    return $d
}

function Get-CargoVersion {
    param([string]$CargoToml)
    $content = Get-Content -Path $CargoToml -Raw
    if ($content -match 'version\s*=\s*"([0-9]+)\.([0-9]+)\.([0-9]+)"') {
        return [int]$Matches[1], [int]$Matches[2], [int]$Matches[3]
    }
    throw "Unable to find version in $CargoToml"
}

function Set-CargoVersion {
    param([string]$CargoToml, [string]$NewVersion)
    (Get-Content -Path $CargoToml -Raw) -replace 'version\s*=\s*"[0-9]+\.[0-9]+\.[0-9]+"', "version = \"$NewVersion\"" | Set-Content -Path $CargoToml -NoNewline
}

function Bump-Version {
    param([int]$Major,[int]$Minor,[int]$Patch,[string]$Kind)
    switch ($Kind) {
        'major' { return "{0}.{1}.{2}" -f ($Major+1), 0, 0 }
        'minor' { return "{0}.{1}.{2}" -f $Major, ($Minor+1), 0 }
        default  { return "{0}.{1}.{2}" -f $Major, $Minor, ($Patch+1) }
    }
}

$Repo = Get-RepoRoot -Start $PSScriptRoot
Set-Location $Repo

# Ensure git available
if (-not (Get-Command git -ErrorAction SilentlyContinue)) { throw 'git is required on PATH.' }

# Stage all current changes
git add -A | Out-Null

# Get current version and compute new version
$CargoToml = Join-Path $Repo 'Cargo.toml'
$v = Get-CargoVersion -CargoToml $CargoToml
$new = Bump-Version -Major $v[0] -Minor $v[1] -Patch $v[2] -Kind $Bump

# Update Cargo.toml
Write-Host "Bumping version to $new" -ForegroundColor Cyan
Set-CargoVersion -CargoToml $CargoToml -NewVersion $new

# Stage version file and commit
git add $CargoToml | Out-Null
$commitMsg = if ([string]::IsNullOrWhiteSpace($Message)) { "chore(release): v$new" } else { "chore(release): v$new - $Message" }
git commit -m "$commitMsg" | Out-Null

# Create annotated tag
$tag = "v$new"
git tag -a $tag -m "$commitMsg"

# Push commit and tag
# Ensure a default remote exists
$remote = (git remote) -split "\r?\n" | Where-Object { $_ -ne '' } | Select-Object -First 1
if (-not $remote) { throw 'No git remote configured. Add a remote and try again.' }

git push $remote HEAD | Out-Null
git push $remote --tags | Out-Null

Write-Host "Pushed $commitMsg and tag $tag to $remote" -ForegroundColor Green
