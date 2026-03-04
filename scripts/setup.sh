#!/bin/bash
# VantisWeb Browser - Setup Script
# This script helps set up the development environment

set -e

echo "🔥 VantisWeb Browser - Setup Script"
echo "=================================="
echo ""

# Check operating system
OS="$(uname -s)"
case "${OS}" in
    Linux*)     MACHINE=Linux;;
    Darwin*)    MACHINE=Mac;;
    CYGWIN*)    MACHINE=Cygwin;;
    MINGW*)     MACHINE=MinGw;;
    *)          MACHINE="UNKNOWN:${OS}"
esac
echo "🖥️  Detected OS: ${MACHINE}"
echo ""

# Check Rust installation
echo "🔧 Checking Rust installation..."
if command -v cargo &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo "✅ Rust is installed: ${RUST_VERSION}"
else
    echo "❌ Rust is not installed!"
    echo "Please install Rust from: https://www.rust-lang.org/tools/install"
    exit 1
fi
echo ""

# Check Rust version
echo "📋 Checking Rust version..."
RUST_MIN_VERSION="1.75.0"
RUST_CURRENT_VERSION=$(rustc --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)

if [ "$(printf '%s\n' "$RUST_MIN_VERSION" "$RUST_CURRENT_VERSION" | sort -V | head -n1)" = "$RUST_MIN_VERSION" ]; then
    echo "✅ Rust version is compatible: ${RUST_CURRENT_VERSION} (required: ${RUST_MIN_VERSION}+)"
else
    echo "⚠️  Rust version might be outdated: ${RUST_CURRENT_VERSION} (recommended: ${RUST_MIN_VERSION}+)"
    echo "Run: rustup update"
fi
echo ""

# Install system dependencies based on OS
echo "📦 Installing system dependencies..."
if [ "${MACHINE}" = "Linux" ]; then
    if command -v apt-get &> /dev/null; then
        echo "🐧 Installing dependencies for Linux (Debian/Ubuntu)..."
        sudo apt-get update
        sudo apt-get install -y \
            libwebkit2gtk-4.0-dev \
            build-essential \
            curl \
            wget \
            file \
            libssl-dev \
            pkg-config
        echo "✅ Linux dependencies installed"
    elif command -v dnf &> /dev/null; then
        echo "🐧 Installing dependencies for Linux (Fedora)..."
        sudo dnf install -y \
            webkit2gtk3-devel \
            gcc \
            gcc-c++ \
            make \
            curl \
            wget \
            openssl-devel \
            pkg-config
        echo "✅ Linux dependencies installed"
    else
        echo "⚠️  Package manager not recognized. Please install webkit2gtk manually."
    fi
elif [ "${MACHINE}" = "Mac" ]; then
    echo "🍎 Installing dependencies for macOS..."
    if command -v brew &> /dev/null; then
        brew install webkit2gtk
        echo "✅ macOS dependencies installed"
    else
        echo "⚠️  Homebrew not found. Install from: https://brew.sh"
    fi
else
    echo "⚠️  Unsupported OS for automatic dependency installation"
fi
echo ""

# Install cargo tools
echo "🔨 Installing useful cargo tools..."
CARGO_TOOLS=(
    "cargo-audit"
    "cargo-outdated"
    "cargo-tarpaulin"
    "cargo-watch"
)

for tool in "${CARGO_TOOLS[@]}"; do
    if ! command -v "$tool" &> /dev/null; then
        echo "Installing $tool..."
        cargo install "$tool"
    else
        echo "✅ $tool is already installed"
    fi
done
echo ""

# Create necessary directories
echo "📁 Creating necessary directories..."
mkdir -p target/release
mkdir -p logs
mkdir -p profiles
echo "✅ Directories created"
echo ""

# Build the project
echo "🏗️  Building VantisWeb..."
cargo build --release
echo "✅ Build completed successfully"
echo ""

# Run tests
echo "🧪 Running tests..."
cargo test
echo "✅ All tests passed"
echo ""

# Generate documentation
echo "📚 Generating documentation..."
cargo doc --no-deps
echo "✅ Documentation generated"
echo ""

echo "🎉 Setup completed successfully!"
echo ""
echo "Next steps:"
echo "  1. Run the browser: cargo run --release"
echo "  2. Read the documentation: open target/doc/index.html"
echo "  3. Check the README: cat README.md"
echo ""
echo "Happy coding! 🚀"