<#
.SYNOPSIS
    Packages Windhawk Windows 10 Notes Exporter into portable ZIP and WiX MSI installers.

.DESCRIPTION
    Builds the release target, verifies the binary, creates a portable ZIP bundle,
    and attempts WiX MSI compilation if WiX Toolset / cargo-wix is installed.
#>

[CmdletBinding()]
param(
    [switch]$SkipBuild,
    [string]$OutputDir = "./target/package"
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$rootDir = (Resolve-Path "$scriptDir/..").Path
Set-Location $rootDir

Write-Host "=== Packaging Windhawk Windows 10 Notes Exporter ===" -ForegroundColor Cyan

if (-not $SkipBuild) {
    Write-Host "Building release binary..." -ForegroundColor Yellow
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        throw "Cargo release build failed."
    }
}

$binPath = "$rootDir/target/release/windhawk-win10.exe"
if (-not (Test-Path $binPath)) {
    throw "Release binary not found at $binPath"
}

# 1. Create Portable Distribution Folder
$distDir = "$rootDir/target/portable_dist"
if (Test-Path $distDir) {
    Remove-Item -Recurse -Force $distDir
}
New-Item -ItemType Directory -Path $distDir -Force | Out-Null
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

Copy-Item $binPath $distDir/
Copy-Item "$rootDir/README.md" $distDir/
Copy-Item "$rootDir/LICENSE" $distDir/

$zipTarget = "$OutputDir/windhawk-win10-portable.zip"
if (Test-Path $zipTarget) {
    Remove-Item -Force $zipTarget
}

Write-Host "Creating portable ZIP archive: $zipTarget" -ForegroundColor Yellow
Compress-Archive -Path "$distDir/*" -DestinationPath $zipTarget -Force
Write-Host "Portable archive created successfully: $zipTarget" -ForegroundColor Green

# 2. Build WiX MSI if tools are present
$candle = Get-Command "candle.exe" -ErrorAction SilentlyContinue
$light = Get-Command "light.exe" -ErrorAction SilentlyContinue

if ($candle -and $light) {
    Write-Host "WiX Toolset detected. Building MSI installer..." -ForegroundColor Yellow
    $wixObj = "$rootDir/target/wix_main.wixobj"
    $msiTarget = "$OutputDir/windhawk-win10-installer.msi"
    
    & $candle.Source -nologo -out $wixObj "$rootDir/wix/main.wxs"
    & $light.Source -nologo -ext WixUIExtension -out $msiTarget $wixObj
    
    if (Test-Path $msiTarget) {
        Write-Host "MSI installer created successfully: $msiTarget" -ForegroundColor Green
    }
} else {
    Write-Host "WiX Toolset (candle.exe / light.exe) not found in PATH." -ForegroundColor Gray
    Write-Host "To generate the MSI package, install WiX Toolset v3 or run 'cargo install cargo-wix'." -ForegroundColor Gray
}

Write-Host "Packaging complete." -ForegroundColor Green
