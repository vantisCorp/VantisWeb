#!/bin/bash
# VantisWeb Browser - Release Script
# Automated release creation and publishing

set -e

echo "🚀 VantisWeb Browser - Release Script"
echo "======================================"
echo ""

# Check if we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "${CURRENT_BRANCH}" != "main" ]; then
    echo "❌ You must be on the 'main' branch to create a release"
    echo "Current branch: ${CURRENT_BRANCH}"
    exit 1
fi

# Check if working tree is clean
if [ -n "$(git status --porcelain)" ]; then
    echo "❌ Working tree is not clean. Please commit or stash changes first."
    git status --short
    exit 1
fi

# Get current version from Cargo.toml
VERSION=$(grep -m1 '^version' Cargo.toml | awk '{print $3}' | tr -d '"')
echo "📋 Current version: ${VERSION}"
echo ""

# Confirm release
read -p "Do you want to create release for version ${VERSION}? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Release cancelled"
    exit 0
fi

echo ""
echo "🏗️  Preparing release ${VERSION}..."
echo ""

# Build release binaries
echo "📦 Building release binaries..."
cargo build --release

# Check if binary exists
if [ ! -f "target/release/vantisweb" ]; then
    echo "❌ Release binary not found!"
    exit 1
fi

# Get binary info
SIZE=$(du -h target/release/vantisweb | cut -f1)
echo "✅ Binary size: ${SIZE}"
echo ""

# Run tests
echo "🧪 Running tests..."
cargo test
echo "✅ All tests passed"
echo ""

# Run linting
echo "🔍 Running linters..."
cargo clippy -- -D warnings
echo "✅ No warnings found"
echo ""

# Check formatting
echo "📝 Checking formatting..."
cargo fmt -- --check
echo "✅ Code is properly formatted"
echo ""

# Create release notes template
echo "📝 Creating release notes..."
cat > "RELEASE_NOTES_${VERSION}.md" << EOF
# VantisWeb Browser v${VERSION}

## 📋 Release Information

**Release Date:** $(date +%Y-%m-%d)
**Version:** ${VERSION}

## ✨ New Features

- Feature 1
- Feature 2

## 🐛 Bug Fixes

- Bug fix 1
- Bug fix 2

## 🔧 Improvements

- Improvement 1
- Improvement 2

## 🔒 Security

- Security update 1

## 📊 Performance

- Performance improvement 1

## 📚 Documentation

- Documentation update 1

## 🔄 Migration Guide

If upgrading from previous version:
- Migration step 1
- Migration step 2

## ⚠️ Known Issues

- Known issue 1
- Known issue 2

## 🙏 Acknowledgments

Thanks to all contributors!

---

[Full Changelog](CHANGELOG.md)
EOF

echo "✅ Release notes template created: RELEASE_NOTES_${VERSION}.md"
echo ""

# Create tag
echo "🏷️  Creating git tag..."
git tag -a "v${VERSION}" -m "Release v${VERSION}"
echo "✅ Tag v${VERSION} created"
echo ""

# Ask if user wants to push
read -p "Do you want to push to GitHub and create release? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo "📤 Pushing to GitHub..."
    
    # Push tag
    git push origin main
    git push origin "v${VERSION}"
    
    echo "✅ Pushed to GitHub"
    echo ""
    echo "🎉 Release v${VERSION} prepared!"
    echo ""
    echo "Next steps:"
    echo "  1. Edit release notes: RELEASE_NOTES_${VERSION}.md"
    echo "  2. Create GitHub release: gh release create v${VERSION} --notes-file RELEASE_NOTES_${VERSION}.md"
    echo "  3. Monitor CI/CD pipeline: gh run list"
else
    echo ""
    echo "✨ Release prepared locally!"
    echo ""
    echo "To push later:"
    echo "  git push origin main"
    echo "  git push origin v${VERSION}"
    echo "  gh release create v${VERSION} --notes-file RELEASE_NOTES_${VERSION}.md"
fi