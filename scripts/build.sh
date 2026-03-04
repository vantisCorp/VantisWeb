#!/bin/bash
# VantisWeb Browser - Build Script
# Automated build with optimizations

set -e

echo "🏗️  VantisWeb Browser - Build Script"
echo "===================================="
echo ""

# Parse arguments
BUILD_TYPE="debug"
RUN_AFTER_BUILD=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            BUILD_TYPE="release"
            shift
            ;;
        --run)
            RUN_AFTER_BUILD=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --clean)
            echo "🧹 Cleaning build artifacts..."
            cargo clean
            echo "✅ Clean completed"
            exit 0
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --release    Build in release mode (optimized)"
            echo "  --run        Run the browser after build"
            echo "  --verbose    Show verbose output"
            echo "  --clean      Clean build artifacts"
            echo "  --help       Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Display build configuration
echo "📋 Build Configuration:"
echo "  Type: ${BUILD_TYPE}"
echo "  Verbose: ${VERBOSE}"
echo "  Run after build: ${RUN_AFTER_BUILD}"
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed!"
    exit 1
fi

# Build
echo "🔨 Building VantisWeb..."
if [ "${BUILD_TYPE}" = "release" ]; then
    if [ "${VERBOSE}" = true ]; then
        cargo build --release --verbose
    else
        cargo build --release
    fi
    echo "✅ Release build completed"
    BINARY="target/release/vantisweb"
else
    if [ "${VERBOSE}" = true ]; then
        cargo build --verbose
    else
        cargo build
    fi
    echo "✅ Debug build completed"
    BINARY="target/debug/vantisweb"
fi
echo ""

# Get binary size
if [ -f "${BINARY}" ]; then
    SIZE=$(du -h "${BINARY}" | cut -f1)
    echo "📊 Binary size: ${SIZE}"
    echo ""
else
    echo "❌ Binary not found!"
    exit 1
fi

# Run if requested
if [ "${RUN_AFTER_BUILD}" = true ]; then
    echo "🚀 Running VantisWeb..."
    echo ""
    "${BINARY}"
else
    echo "✨ Build completed successfully!"
    echo ""
    echo "To run the browser:"
    echo "  ${BINARY}"
    echo ""
    echo "Or use:"
    echo "  cargo run --${BUILD_TYPE}"
fi