#!/bin/bash
set -e

echo "📦 Building RPM package for VantisWeb"
echo "======================================"
echo ""

# Get version from Cargo.toml
VERSION=$(grep -m1 '^version' ../../Cargo.toml | awk '{print $3}' | tr -d '"')
PACKAGE_NAME="vantisweb"
PACKAGE_VERSION="${VERSION}"

echo "📋 Package Information:"
echo "  Name: ${PACKAGE_NAME}"
echo "  Version: ${PACKAGE_VERSION}"
echo ""

# Check if rpmbuild is installed
if ! command -v rpmbuild &> /dev/null; then
    echo "❌ rpmbuild not found!"
    echo "Install it with: sudo dnf install rpm-build OR sudo yum install rpm-build"
    exit 1
fi

# Create tarball
echo "📦 Creating source tarball..."
mkdir -p rpmbuild/SOURCES
tar czf rpmbuild/SOURCES/${PACKAGE_NAME}-${PACKAGE_VERSION}.tar.gz \
    --exclude=target \
    --exclude=.git \
    --exclude=*.deb \
    --exclude=*.rpm \
    ../..

# Copy spec file
mkdir -p rpmbuild/SPECS
cp ../../packaging/linux/rpm/vantisweb.spec rpmbuild/SPECS/

# Build RPM
echo "🔨 Building RPM package..."
rpmbuild -ba rpmbuild/SPECS/vantisweb.spec \
    --define "_topdir $(pwd)/rpmbuild" \
    --define "VERSION ${PACKAGE_VERSION}"

# Find built packages
RPMS=$(find rpmbuild/RPMS -name "*.rpm" 2>/dev/null)

if [ -z "$RPMS" ]; then
    echo "❌ RPM package build failed!"
    exit 1
fi

# Copy packages to current directory
for rpm in $RPMS; do
    cp "$rpm" ./
    echo "✅ Built: $(basename $rpm)"
done

# Cleanup
rm -rf rpmbuild

echo ""
echo "✅ RPM packages built successfully!"
echo ""
echo "To install:"
echo "  sudo dnf install ${PACKAGE_NAME}-${PACKAGE_VERSION}*.rpm"
echo "  OR"
echo "  sudo yum install ${PACKAGE_NAME}-${PACKAGE_VERSION}*.rpm"
echo ""
