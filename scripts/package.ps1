param(
    [string]$Profile = "dev"
)

$ErrorActionPreference = "Stop"

# Resolve repo root
$RepoRoot = Split-Path -Parent $PSScriptRoot
$Dist = Join-Path $RepoRoot 'dist'
$Target = Join-Path $RepoRoot 'target' 'release'

# Ensure dist directory exists
if (-not (Test-Path $Dist)) { New-Item -ItemType Directory -Path $Dist | Out-Null }

Write-Host "Building release binary..." -ForegroundColor Cyan
& "C:\Program Files\Rust stable MSVC 1.88\bin\cargo.exe" build --release

# Determine app name and version
$cargoToml = Get-Content -Path (Join-Path $RepoRoot 'Cargo.toml') -Raw
if ($cargoToml -match 'name\s*=\s*"([^"]+)"') { $AppName = $Matches[1] } else { $AppName = 'app' }
if ($cargoToml -match 'version\s*=\s*"([^"]+)"') { $Version = $Matches[1] } else { $Version = '0.0.0' }

$ExePath = Join-Path $Target ("$AppName.exe")
if (-not (Test-Path $ExePath)) {
    throw "Executable not found at $ExePath. Build may have failed."
}

$ZipName = "$AppName-$Version-windows-x64.zip"
$ZipPath = Join-Path $Dist $ZipName

# Copy license and README for distribution
$TempDir = Join-Path $Dist "$AppName-$Version-windows-x64"
if (Test-Path $TempDir) { Remove-Item -Recurse -Force $TempDir }
New-Item -ItemType Directory -Path $TempDir | Out-Null
Copy-Item $ExePath $TempDir
if (Test-Path (Join-Path $RepoRoot 'README.md')) { Copy-Item (Join-Path $RepoRoot 'README.md') $TempDir }
if (Test-Path (Join-Path $RepoRoot 'LICENSE')) { Copy-Item (Join-Path $RepoRoot 'LICENSE') $TempDir }

# Create zip
if (Test-Path $ZipPath) { Remove-Item $ZipPath -Force }
Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::CreateFromDirectory($TempDir, $ZipPath)

Write-Host "Packaged => $ZipPath" -ForegroundColor Green
