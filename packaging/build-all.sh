#!/bin/bash
set -e

echo "🚀 Building All VantisWeb Installers"
echo "====================================="
echo ""

# Build release binary first
echo "🔨 Building release binary..."
cargo build --release

echo ""
echo "📦 Building packages..."
echo ""

# Build DEB
if [ -d "packaging/linux/deb" ]; then
    echo "Building DEB package..."
    cd packaging/linux/deb
    ./build-deb.sh || echo "⚠️  DEB build skipped (requires dependencies)"
    cd ../..
fi

# Build RPM
if [ -d "packaging/linux/rpm" ]; then
    echo "Building RPM package..."
    cd packaging/linux/rpm
    ./build-rpm.sh || echo "⚠️  RPM build skipped (requires rpmbuild)"
    cd ../..
fi

echo ""
echo "✅ Build process completed!"
echo ""
echo "Available packages:"
ls -lh *.deb *.rpm 2>/dev/null || echo "No packages built yet"
