# VantisWeb Browser - Installation Packages

This directory contains all files needed to build installation packages for VantisWeb Browser.

## 📦 Available Packages

### Linux

#### Debian/Ubuntu (DEB)
- **File**: `vantisweb_VERSION_amd64.deb`
- **Compatible**: Debian 10+, Ubuntu 20.04+
- **Installation**: `sudo dpkg -i vantisweb_*.deb`

#### Fedora/RedHat (RPM)
- **File**: `vantisweb-VERSION-1.x86_64.rpm`
- **Compatible**: Fedora 35+, RHEL 8+, CentOS 8+
- **Installation**: `sudo dnf install vantisweb-*.rpm`

### Windows
- **File**: `vantisweb.exe` (installer) or `vantisweb.zip` (portable)
- **Compatible**: Windows 10, Windows 11
- **Installation**: Double-click the installer

### macOS
- **File**: `VantisWeb.dmg`
- **Compatible**: macOS 10.15+, macOS 11+, macOS 12+
- **Installation**: Open DMG and drag to Applications

## 🔧 Building Packages Locally

### Prerequisites

**Linux:**
```bash
# Debian/Ubuntu
sudo apt-get install build-essential cargo dpkg-dev fakeroot

# Fedora/RedHat
sudo dnf install cargo gcc make rpm-build
```

**Windows:**
- Install Rust from https://rustup.rs/
- Install WiX Toolset for MSI creation

**macOS:**
- Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

### Build Commands

**Linux DEB:**
```bash
cd packaging/linux/deb
./build-deb.sh
```

**Linux RPM:**
```bash
cd packaging/linux/rpm
./build-rpm.sh
```

**Windows:**
```bash
cd packaging/windows
./build-windows.sh
```

**macOS:**
```bash
cd packaging/macos
./build-macos.sh
```

**All platforms:**
```bash
# Build release binary first
cd ../..
cargo build --release

# Then build packages
cd packaging
./build-all.sh
```

## 🚀 Automated Builds

All packages are automatically built via GitHub Actions when:
- A new release tag is pushed (e.g., `v0.1.0`)
- Code is merged to `main` branch
- Manual trigger via GitHub Actions UI

Download releases from: https://github.com/vantisCorp/VantisWeb/releases

## 📋 Package Contents

All packages include:
- VantisWeb browser binary
- Desktop integration (menu entry, icons)
- Application shortcuts
- Dependencies (Linux packages)
- Uninstall scripts

## 🗑️ Uninstallation

### Linux DEB:
```bash
sudo apt-get remove vantisweb
```

### Linux RPM:
```bash
sudo dnf remove vantisweb
# or
sudo yum remove vantisweb
```

### Windows:
- Use "Add or Remove Programs" in Windows Settings
- Or run uninstaller from Start Menu

### macOS:
```bash
rm -rf /Applications/VantisWeb.app
```

## 🔍 Troubleshooting

### Linux
**Package not found error:**
```bash
# Update package lists
sudo apt-get update  # Debian/Ubuntu
sudo dnf update      # Fedora/RedHat

# Install dependencies
sudo apt-get install -f  # Debian/Ubuntu
```

**Missing dependencies:**
- Ensure your system has GTK3 and WebKitGTK installed
- Check package manager logs for specific errors

### Windows
**Virus scanner warning:**
- Windows Defender may flag unsigned binaries
- Allow the file through your antivirus
- Only download from official GitHub releases

### macOS
**App won't open:**
- Right-click → Open to bypass Gatekeeper
- Or: `sudo xattr -rd com.apple.quarantine /Applications/VantisWeb.app`

## 📝 Signing Packages

To sign packages for distribution:

**Linux DEB/RPM:**
```bash
# DEB
debsign vantisweb_*.deb

# RPM
rpmsign --addsign vantisweb-*.rpm
```

**Windows:**
```bash
signtool sign /f certificate.pfx /p password vantisweb.exe
```

**macOS:**
```bash
codesign --sign "Developer ID" VantisWeb.app
```

## 🤝 Contributing

When adding new platform-specific files:
1. Update build scripts
2. Update this README
3. Test on target platform
4. Update GitHub Actions workflow

## 📞 Support

For issues with installers:
- Open an issue: https://github.com/vantisCorp/VantisWeb/issues
- Check documentation: https://github.com/vantisCorp/VantisWeb/wiki

---

**VantisWeb Browser** - Next-generation web browsing experience
© 2024 Vantis Corp
