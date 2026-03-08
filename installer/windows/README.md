# VantisWeb Windows Installer

This directory contains the Windows installer configuration for VantisWeb Browser.

## Files

| File | Description |
|------|-------------|
| `installer.nsi` | NSIS installer script |
| `build-installer.ps1` | PowerShell build script |
| `setup_wizard.rs` | Setup wizard component |

## Prerequisites

To build the Windows installer, you need:

1. **Rust** - Install from [rustup.rs](https://rustup.rs/)
2. **NSIS** - Install via chocolatey: `choco install nsis`
3. **Visual Studio Build Tools** - For Windows SDK

## Building

### Using PowerShell Script

```powershell
# Build with default settings
.\build-installer.ps1

# Build specific version
.\build-installer.ps1 -Version "1.5.0"

# Build and sign
.\build-installer.ps1 -Sign -CertificatePath "cert.pfx" -CertificatePassword "pass"
```

### Manual Build

1. Build the release executable:
```bash
cargo build --release
```

2. Build the installer with NSIS:
```bash
makensis installer/windows/installer.nsi
```

## Installer Features

### Installation Options

- **Desktop Shortcut** - Create shortcut on desktop
- **Start Menu** - Add to Start Menu programs
- **Default Browser** - Set VantisWeb as default browser
- **Taskbar Pin** - Pin to Windows taskbar

### File Associations

The installer registers these file types:
- `.html` - HTML documents
- `.htm` - HTM documents
- `.xhtml` - XHTML documents

### Protocol Handlers

- `http://` - HTTP protocol
- `https://` - HTTPS protocol

## Silent Installation

For unattended installation:

```powershell
# Silent install with default options
VantisWeb-Setup-1.5.0.exe /S

# Silent install with specific options
VantisWeb-Setup-1.5.0.exe /S /D=C:\Custom\Path
```

### Silent Install Parameters

| Parameter | Description |
|-----------|-------------|
| `/S` | Silent mode |
| `/D=path` | Custom installation path |
| `/NOICONS` | Skip desktop shortcuts |
| `/NODEFAULTBROWSER` | Don't set as default browser |
| `/NOLAUNCH` | Don't launch after install |

## Portable Mode

VantisWeb also supports portable mode:

1. Download the portable ZIP package
2. Extract to any location
3. Create `portable.ini` file in the application directory
4. Run `VantisWeb.exe`

## Uninstallation

Uninstall via:
- Start Menu → VantisWeb → Uninstall
- Settings → Apps → VantisWeb → Uninstall
- Run `C:\Program Files\VantisWeb\Uninstall.exe`

## Customization

### Custom Branding

Replace these files for custom branding:
- `assets/icons/icon.ico` - Application icon
- `assets/installer/welcome.bmp` - Welcome page banner
- `assets/installer/header.bmp` - Header image

### Adding Components

Edit `installer.nsi` to add additional components:

```nsis
Section "My Component" SecMyComponent
    File /r "my-component\*.*"
SectionEnd
```

## Code Signing

To sign the installer:

1. Obtain a code signing certificate
2. Build with the `-Sign` parameter:

```powershell
.\build-installer.ps1 -Sign -CertificatePath "cert.pfx" -CertificatePassword "password"
```

## Troubleshooting

### Installer won't run

- Run as Administrator
- Check Windows Defender SmartScreen
- Verify the installer is not corrupted (check SHA256)

### Installation fails

- Check available disk space (200MB minimum)
- Close running instances of VantisWeb
- Check write permissions for installation directory

### Browser not appearing as default

- Run the installer again with "Set as Default Browser" option
- Manually set via Windows Settings → Apps → Default apps

## Support

- GitHub Issues: https://github.com/vantisCorp/VantisWeb/issues
- Documentation: https://docs.vantis.ai
- Email: support@vantis.ai