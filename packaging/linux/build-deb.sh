#!/bin/bash
set -e

echo "📦 Building DEB package for VantisWeb"
echo "======================================"
echo ""

# Get version from Cargo.toml
VERSION=$(grep -m1 '^version' ../../Cargo.toml | awk '{print $3}' | tr -d '"')
ARCH="amd64"
PACKAGE_NAME="vantisweb"
PACKAGE_VERSION="${VERSION}"

echo "📋 Package Information:"
echo "  Name: ${PACKAGE_NAME}"
echo "  Version: ${PACKAGE_VERSION}"
echo "  Architecture: ${ARCH}"
echo ""

# Clean previous builds
rm -rf deb-build
mkdir -p deb-build/DEBIAN
mkdir -p deb-build/usr/bin
mkdir -p deb-build/usr/share/applications
mkdir -p deb-build/usr/share/icons/hicolor/256x256/apps
mkdir -p deb-build/opt/vantisweb
mkdir -p deb-build/lib/systemd/system

# Copy control files
echo "📝 Copying control files..."
cp control deb-build/DEBIAN/
cp postinst deb-build/DEBIAN/
chmod 755 deb-build/DEBIAN/postinst

# Copy desktop entry
echo "🖥️  Copying desktop entry..."
cp ../../packaging/linux/vantisweb.desktop deb-build/usr/share/applications/

# Copy binary (will be replaced by actual build)
echo "🔧 Copying binary..."
if [ -f "../../target/release/vantisweb" ]; then
    cp ../../target/release/vantisweb deb-build/usr/bin/
    chmod 755 deb-build/usr/bin/vantisweb
else
    echo "⚠️  Warning: Release binary not found!"
    echo "Please run: cargo build --release"
    exit 1
fi

# Create icon (placeholder - should be replaced with actual icon)
echo "🎨 Creating icon..."
cat > deb-build/usr/share/icons/hicolor/256x256/apps/vantisweb.png << 'ICON_EOF'
# This is a placeholder - replace with actual icon file
# You can convert SVG to PNG using: convert icon.svg -resize 256x256 vantisweb.png
echo "Icon placeholder - please add actual icon"
ICON_EOF

# Create directories for application data
mkdir -p deb-build/opt/vantisweb
mkdir -p deb-build/opt/vantisweb/extensions
mkdir -p deb-build/opt/vantisweb/profiles
mkdir -p deb-build/opt/vantisweb/cache

# Copy application resources if they exist
if [ -d "../../assets" ]; then
    cp -r ../../assets/* deb-build/opt/vantisweb/ 2>/dev/null || true
fi

# Create symlink
ln -sf /usr/bin/vantisweb deb-build/opt/vantisweb/vantisweb

# Calculate installed size
INSTALLED_SIZE=$(du -sk deb-build | cut -f1)
sed -i "s/^Installed-Size:.*/Installed-Size: ${INSTALLED_SIZE}/" deb-build/DEBIAN/control

# Build the package
echo ""
echo "🔨 Building DEB package..."
cd deb-build
dpkg-deb --build . "../${PACKAGE_NAME}_${PACKAGE_VERSION}_${ARCH}.deb"
cd ..

# Cleanup
rm -rf deb-build

echo ""
echo "✅ DEB package built successfully!"
echo "📦 Package: ${PACKAGE_NAME}_${PACKAGE_VERSION}_${ARCH}.deb"
echo ""
echo "To install:"
echo "  sudo dpkg -i ${PACKAGE_NAME}_${PACKAGE_VERSION}_${ARCH}.deb"
echo ""
echo "Or to fix dependencies:"
echo "  sudo apt-get install -f"
echo ""
