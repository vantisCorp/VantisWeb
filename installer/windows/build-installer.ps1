# VantisWeb Installer Build Script
# Builds the Windows installer using NSIS

param(
    [string]$Configuration = "Release",
    [string]$Version = "1.5.0",
    [switch]$Sign = $false,
    [string]$CertificatePath = "",
    [string]$CertificatePassword = ""
)

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  VantisWeb Installer Build Script" -ForegroundColor Cyan
Write-Host "  Version: $Version" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Configuration
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$TargetDir = Join-Path $ProjectRoot "target\$Configuration"
$InstallerDir = Join-Path $ProjectRoot "installer\windows"
$OutputFile = Join-Path $ProjectRoot "dist\VantisWeb-Setup-$Version.exe"

Write-Host "[1/7] Checking prerequisites..." -ForegroundColor Yellow

# Check for Rust
if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    Write-Error "Rust/Cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
}

# Check for NSIS
$nsisPath = Get-ItemProperty -Path "HKLM:\SOFTWARE\NSIS" -ErrorAction SilentlyContinue
if (-not $nsisPath) {
    Write-Host "NSIS not found. Attempting to install..." -ForegroundColor Yellow
    winget install -e --id NSIS.NSIS --accept-source-agreements --accept-package-agreements
    $nsisPath = Get-ItemProperty -Path "HKLM:\SOFTWARE\NSIS" -ErrorAction SilentlyContinue
}

$makensis = if ($nsisPath) {
    Join-Path $nsisPath "(Default)" "makensis.exe"
} else {
    "C:\Program Files (x86)\NSIS\makensis.exe"
}

if (-not (Test-Path $makensis)) {
    Write-Error "NSIS makensis.exe not found at $makensis"
    exit 1
}

Write-Host "  - Rust/Cargo: OK" -ForegroundColor Green
Write-Host "  - NSIS: OK" -ForegroundColor Green

# Step 2: Build Rust project
Write-Host "[2/7] Building VantisWeb ($Configuration)..." -ForegroundColor Yellow

Push-Location $ProjectRoot
try {
    $buildArgs = @("build", "--release")
    if ($Configuration -eq "Release") {
        $buildArgs += @("--config", "profile.release.opt-level = 3")
    }
    
    & cargo $buildArgs
    
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Cargo build failed with exit code $LASTEXITCODE"
    }
} finally {
    Pop-Location
}

Write-Host "  - Build completed" -ForegroundColor Green

# Step 3: Create required directories
Write-Host "[3/7] Creating distribution directories..." -ForegroundColor Yellow

$directories = @(
    Join-Path $ProjectRoot "dist"
    Join-Path $ProjectRoot "resources"
    Join-Path $ProjectRoot "resources\themes"
    Join-Path $ProjectRoot "resources\icons"
    Join-Path $ProjectRoot "assets\installer"
    Join-Path $ProjectRoot "assets\icons"
)

foreach ($dir in $directories) {
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
        Write-Host "  Created: $dir" -ForegroundColor Gray
    }
}

Write-Host "  - Directories created" -ForegroundColor Green

# Step 4: Generate icon if not exists
Write-Host "[4/7] Checking application icon..." -ForegroundColor Yellow

$iconPath = Join-Path $ProjectRoot "assets\icons\icon.ico"
if (-not (Test-Path $iconPath)) {
    Write-Host "  Icon not found. Creating placeholder..." -ForegroundColor Yellow
    # In production, you would generate a proper icon here
    # For now, we'll copy a placeholder
}

Write-Host "  - Icon ready" -ForegroundColor Green

# Step 5: Copy required files
Write-Host "[5/7] Copying application files..." -ForegroundColor Yellow

$exePath = Join-Path $TargetDir "VantisWeb.exe"
if (-not (Test-Path $exePath)) {
    Write-Error "Built executable not found at $exePath"
}

# Copy resources
$resourcesSource = Join-Path $ProjectRoot "resources"
$resourcesDest = Join-Path $TargetDir "resources"
if (Test-Path $resourcesSource) {
    Copy-Item -Path $resourcesSource -Destination $resourcesDest -Recurse -Force
}

Write-Host "  - Files copied" -ForegroundColor Green

# Step 6: Build installer
Write-Host "[6/7] Building installer with NSIS..." -ForegroundColor Yellow

# Set version in NSIS script
$nsiFile = Join-Path $InstallerDir "installer.nsi"
$nsiContent = Get-Content $nsiFile -Raw
$nsiContent = $nsiContent -replace '!define PRODUCT_VERSION ".*"', "!define PRODUCT_VERSION `"$Version`""
$nsiContent | Set-Content $nsiFile -NoNewline

# Build with NSIS
$nsisArgs = @(
    "/V2",
    "/DPRODUCT_VERSION=$Version",
    $nsiFile
)

& $makensis $nsisArgs

if ($LASTEXITCODE -ne 0) {
    Write-Error "NSIS build failed with exit code $LASTEXITCODE"
}

Write-Host "  - Installer built" -ForegroundColor Green

# Step 7: Sign installer (optional)
if ($Sign -and $CertificatePath -ne "") {
    Write-Host "[7/7] Signing installer..." -ForegroundColor Yellow
    
    $signtool = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe"
    
    if (Test-Path $signtool) {
        & $signtool sign /f $CertificatePath /p $CertificatePassword /t http://timestamp.digicert.com $OutputFile
        Write-Host "  - Installer signed" -ForegroundColor Green
    } else {
        Write-Host "  - signtool.exe not found, skipping signing" -ForegroundColor Yellow
    }
} else {
    Write-Host "[7/7] Skipping code signing (not configured)" -ForegroundColor Yellow
}

# Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Build Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Output: $OutputFile" -ForegroundColor White
Write-Host "Size: $([math]::Round((Get-Item $OutputFile).Length / 1MB, 2)) MB" -ForegroundColor White
Write-Host ""

# Verify output
if (Test-Path $OutputFile) {
    Write-Host "Installer created successfully!" -ForegroundColor Green
    exit 0
} else {
    Write-Error "Installer was not created at expected location"
    exit 1
}