#Requires -Version 5.1
<#
.SYNOPSIS
    Build TimeTracker for Windows.

.PARAMETER Release
    Build in release mode (default). Pass -Release:$false for a debug build.

.PARAMETER Target
    Rust target triple. Defaults to x86_64-pc-windows-msvc.

.PARAMETER Bundle
    What to produce:
      exe   - raw .exe only, no installer (fastest, default)
      nsis  - NSIS setup .exe installer
      msi   - MSI installer
      all   - both NSIS and MSI installers

.PARAMETER SkipFrontend
    Skip the `npm install` step (useful when deps are already up-to-date).

.EXAMPLE
    .\build-windows.ps1                      # exe only  (default)
    .\build-windows.ps1 -Bundle nsis         # NSIS installer
    .\build-windows.ps1 -Bundle msi          # MSI installer
    .\build-windows.ps1 -Bundle all          # both installers
    .\build-windows.ps1 -Release:$false      # debug exe
    .\build-windows.ps1 -Bundle nsis -SkipFrontend
#>

param(
    [bool]   $Release      = $true,
    [string] $Target       = "x86_64-pc-windows-msvc",
    [string] $Bundle       = "exe",
    [switch] $SkipFrontend
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# -- Helpers --

function Write-Step([string]$msg) {
    Write-Host ""
    Write-Host "  >> $msg" -ForegroundColor Cyan
}

function Require-Command([string]$cmd) {
    if (-not (Get-Command $cmd -ErrorAction SilentlyContinue)) {
        Write-Host "  ERROR: '$cmd' not found in PATH." -ForegroundColor Red
        exit 1
    }
}

function Get-ElapsedTime([datetime]$start) {
    $elapsed = (Get-Date) - $start
    if ($elapsed.TotalMinutes -ge 1) {
        return "{0}m {1}s" -f [int]$elapsed.TotalMinutes, $elapsed.Seconds
    }
    return "{0}s" -f [int]$elapsed.TotalSeconds
}

# -- Banner --

$scriptStart = Get-Date
$repoRoot    = Split-Path $PSScriptRoot -Parent
$mode        = if ($Release) { "release" } else { "debug" }

Write-Host ""
Write-Host "  +--------------------------------------+" -ForegroundColor DarkCyan
Write-Host "  |   TimeTracker  -  Windows Build      |" -ForegroundColor DarkCyan
Write-Host "  +--------------------------------------+" -ForegroundColor DarkCyan
Write-Host ""
Write-Host "  Repo   : $repoRoot"
Write-Host "  Target : $Target"
Write-Host "  Bundle : $Bundle"
Write-Host "  Mode   : $mode"

# -- Pre-flight checks --

Write-Step "Checking prerequisites"

Require-Command "cargo"
Require-Command "rustup"
Require-Command "npm"

Write-Host "    cargo : $(cargo --version 2>&1)"
Write-Host "    node  : $(node --version 2>&1)"

# Ensure the Rust target is installed
Write-Step "Ensuring Rust target: $Target"
rustup target add $Target
if ($LASTEXITCODE -ne 0) { Write-Host "  Failed to add target." -ForegroundColor Red; exit 1 }

# Locate tauri-cli (prefer local node_modules, fall back to global cargo-tauri)
$tauriCli = $null
$localCli  = Join-Path $repoRoot "node_modules\.bin\tauri.cmd"

if (Test-Path $localCli) {
    $tauriCli = $localCli
    Write-Host "    tauri-cli: local ($localCli)"
} elseif (Get-Command "cargo-tauri" -ErrorAction SilentlyContinue) {
    $tauriCli = "cargo-tauri"
    Write-Host "    tauri-cli: cargo-tauri (global)"
} else {
    Write-Step "Installing tauri-cli via cargo (one-time)"
    cargo install tauri-cli --version "^2" --locked
    if ($LASTEXITCODE -ne 0) { Write-Host "  Failed to install tauri-cli." -ForegroundColor Red; exit 1 }
    $tauriCli = "cargo-tauri"
}

# -- Frontend --

Set-Location $repoRoot

if (-not $SkipFrontend) {
    Write-Step "Installing npm dependencies"
    npm install --prefer-offline
    if ($LASTEXITCODE -ne 0) { Write-Host "  npm install failed." -ForegroundColor Red; exit 1 }
}

# -- Tauri build --

Write-Step "Running Tauri build"

$buildArgs = @("build", "--target", $Target)

if (-not $Release) {
    $buildArgs += "--debug"
}

switch ($Bundle.ToLower()) {
    "exe"  { $buildArgs += "--no-bundle" }          # raw exe, skip packaging
    "nsis" { $buildArgs += @("--bundles", "nsis") }
    "msi"  { $buildArgs += @("--bundles", "msi") }
    "all"  { }                                       # no flag = all configured bundles
    default {
        Write-Host "  Unknown -Bundle value '$Bundle'." -ForegroundColor Red
        Write-Host "  Valid values: exe | nsis | msi | all" -ForegroundColor Red
        exit 1
    }
}

Write-Host "    Command: $tauriCli $($buildArgs -join ' ')"
Write-Host ""

& $tauriCli @buildArgs

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "  BUILD FAILED (exit code $LASTEXITCODE)" -ForegroundColor Red
    exit $LASTEXITCODE
}

# -- Locate output --

Write-Step "Locating build artifacts"

$targetDir = Join-Path $repoRoot "src-tauri\target\$Target\$mode"

$artifacts = @()

if ($Bundle.ToLower() -eq "exe") {
    # --no-bundle: the exe sits directly in the target dir
    $exe = Join-Path $targetDir "timetracker.exe"
    if (Test-Path $exe) {
        $artifacts += Get-Item $exe
    }
} else {
    # Installers go into target/.../bundle/
    $bundleRoot = Join-Path $targetDir "bundle"
    foreach ($ext in @("*.exe", "*.msi")) {
        $artifacts += Get-ChildItem -Path $bundleRoot -Recurse -Filter $ext -ErrorAction SilentlyContinue
    }
}

if ($artifacts.Count -gt 0) {
    Write-Host ""
    Write-Host "  Artifacts:" -ForegroundColor Green
    foreach ($a in $artifacts) {
        $sizeMb = [math]::Round($a.Length / 1MB, 1)
        Write-Host "    $($a.FullName)  ($sizeMb MB)" -ForegroundColor Green
    }
} else {
    Write-Host "    No artifacts found under: $targetDir" -ForegroundColor Yellow
}

# -- Copy artifacts to scripts folder --

Write-Step "Copying artifacts to scripts folder"

$outDir = $PSScriptRoot

foreach ($a in $artifacts) {
    $dest = Join-Path $outDir $a.Name
    Copy-Item -Path $a.FullName -Destination $dest -Force
    $sizeMb = [math]::Round($a.Length / 1MB, 1)
    Write-Host "    $dest  ($sizeMb MB)" -ForegroundColor Green
}

# -- Done --

$elapsed = Get-ElapsedTime $scriptStart
Write-Host ""
Write-Host "  OK Build complete in $elapsed" -ForegroundColor Green
Write-Host ""
