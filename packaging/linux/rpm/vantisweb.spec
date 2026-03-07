Name:           vantisweb
Version:        0.1.0
Release:        1%{?dist}
Summary:        Next-generation web browser with Liquid Core Architecture

License:        MIT
URL:            https://github.com/vantisCorp/VantisWeb
Source0:        %{name}-%{version}.tar.gz

# Build dependencies
BuildRequires:  cargo, rust, gcc, make

# Runtime dependencies
Requires:       glibc, openssl-libs, libX11, libXext, libXrandr, libXrender, libXtst, gtk3, webkit2gtk4.1
Recommends:     libnotify

%description
VantisWeb is a modern, high-performance web browser built with Rust
and featuring advanced security, privacy, and AI-powered features.

Key features:
- Advanced security and privacy protections
- AI-powered ad blocking and content filtering
- Built-in password manager and VPN
- Developer tools and extensions support
- Cross-platform support (Linux, Windows, macOS)
- Fast and lightweight using Liquid Core Architecture
- Modern UI with customizable themes

%prep
%autosetup -n %{name}-%{version}

%build
cargo build --release

%install
# Create directories
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_datadir}/applications
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/256x256/apps
mkdir -p %{buildroot}%{_datadir}/doc/%{name}
mkdir -p %{buildroot}%{_sysconfdir}/%{name}

# Install binary
install -m 755 target/release/vantisweb %{buildroot}%{_bindir}/vantisweb

# Install desktop file
install -m 644 packaging/linux/vantisweb.desktop %{buildroot}%{_datadir}/applications/

# Install icon (placeholder)
# install -m 644 packaging/linux/vantisweb.png %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/vantisweb.png

# Install documentation
install -m 644 README.md %{buildroot}%{_datadir}/doc/%{name}/
install -m 644 LICENSE %{buildroot}%{_datadir}/doc/%{name}/
install -m 644 CHANGELOG.md %{buildroot}%{_datadir}/doc/%{name}/ 2>/dev/null || true

%files
%license LICENSE
%doc README.md CHANGELOG.md
%{_bindir}/vantisweb
%{_datadir}/applications/vantisweb.desktop
%{_datadir}/icons/hicolor/256x256/apps/vantisweb.png
%{_datadir}/doc/%{name}

%post
# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database /usr/share/applications || true
fi

# Update icon cache
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
fi

echo "VantisWeb Browser installed successfully!"
echo "Launch from: Applications -> Internet -> VantisWeb Browser"

%postun
# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database /usr/share/applications || true
fi

# Update icon cache
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
fi

%changelog
* $(date +'%a %b %d %Y') Vantis Corp <info@vantiscorp.com> - %{version}-%{release}
- Initial release
