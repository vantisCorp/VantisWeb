@echo off
REM VantisWeb Installer Build Script
REM Builds the Windows installer using NSIS

setlocal EnableDelayedExpansion

echo ========================================
echo   VantisWeb Installer Build Script
echo ========================================
echo.

REM Check for required tools
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo ERROR: Rust/Cargo not found. Please install from https://rustup.rs/
    exit /b 1
)

where makensis >nul 2>nul
if %errorlevel% neq 0 (
    echo NSIS not found. Attempting to install via chocolatey...
    where choco >nul 2>nul
    if %errorlevel% neq 0 (
        echo ERROR: Chocolatey not found. Please install NSIS manually.
        exit /b 1
    )
    choco install nsis -y
)

REM Get version from Cargo.toml
for /f "tokens=3 delims== " %%a in ('findstr /r "^version" Cargo.toml') do (
    set VERSION=%%a
    set VERSION=!VERSION:"=!
)

echo Version: %VERSION%
echo.

REM Build release executable
echo [1/4] Building release executable...
cargo build --release
if %errorlevel% neq 0 (
    echo ERROR: Build failed!
    exit /b 1
)
echo       Done.
echo.

REM Create directories
echo [2/4] Creating directories...
if not exist dist mkdir dist
if not exist assets\icons mkdir assets\icons
if not exist assets\installer mkdir assets\installer
if not exist resources\themes mkdir resources\themes
echo       Done.
echo.

REM Copy files
echo [3/4] Copying files...
copy /Y target\release\VantisWeb.exe dist\ >nul 2>nul
copy /Y README.md dist\ >nul 2>nul
copy /Y LICENSE dist\ >nul 2>nul
copy /Y CHANGELOG.md dist\ >nul 2>nul
echo       Done.
echo.

REM Build installer
echo [4/4] Building NSIS installer...
makensis /V2 /DPRODUCT_VERSION=%VERSION% installer\windows\installer.nsi
if %errorlevel% neq 0 (
    echo ERROR: NSIS build failed!
    exit /b 1
)

REM Move installer to dist
move /Y VantisWeb-Setup-*.exe dist\ >nul 2>nul
move /Y installer\windows\VantisWeb-Setup-*.exe dist\ >nul 2>nul

echo.
echo ========================================
echo   Build Complete!
echo ========================================
echo.
echo Output: dist\VantisWeb-Setup-%VERSION%.exe
echo.

REM Create checksums
cd dist
for %%f in (VantisWeb-Setup-*.exe) do (
    certutil -hashfile %%f SHA256 > %%f.sha256
)

echo Checksums created in dist\

pause