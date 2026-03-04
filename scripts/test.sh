#!/bin/bash
# VantisWeb Browser - Test Script
# Run all tests with coverage reporting

set -e

echo "🧪 VantisWeb Browser - Test Script"
echo "=================================="
echo ""

# Parse arguments
TEST_TYPE="all"
COVERAGE=false
VERBOSE=false
SPECIFIC_TEST=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --unit)
            TEST_TYPE="unit"
            shift
            ;;
        --integration)
            TEST_TYPE="integration"
            shift
            ;;
        --coverage)
            COVERAGE=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --unit          Run unit tests only"
            echo "  --integration   Run integration tests only"
            echo "  --coverage      Generate coverage report"
            echo "  --verbose       Show verbose output"
            echo "  --help          Show this help message"
            echo ""
            echo "Example:"
            echo "  $0 --unit --coverage    # Run unit tests with coverage"
            echo "  $0 --integration        # Run integration tests"
            exit 0
            ;;
        *)
            # Assume it's a specific test name
            SPECIFIC_TEST="$1"
            shift
            ;;
    esac
done

echo "📋 Test Configuration:"
echo "  Type: ${TEST_TYPE}"
echo "  Coverage: ${COVERAGE}"
echo "  Verbose: ${VERBOSE}"
if [ -n "${SPECIFIC_TEST}" ]; then
    echo "  Specific test: ${SPECIFIC_TEST}"
fi
echo ""

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo is not installed!"
    exit 1
fi

# Run tests based on type
run_tests() {
    local test_args=""
    
    if [ "${VERBOSE}" = true ]; then
        test_args="--nocapture"
    fi
    
    if [ -n "${SPECIFIC_TEST}" ]; then
        echo "🔍 Running specific test: ${SPECIFIC_TEST}"
        cargo test "${SPECIFIC_TEST}" -- ${test_args}
        return
    fi
    
    case "${TEST_TYPE}" in
        "unit")
            echo "🔬 Running unit tests..."
            cargo test --lib -- ${test_args}
            ;;
        "integration")
            echo "🔗 Running integration tests..."
            cargo test --test '*' -- ${test_args}
            ;;
        "all")
            echo "🧪 Running all tests..."
            cargo test -- ${test_args}
            ;;
    esac
}

# Run coverage if requested
run_coverage() {
    echo ""
    echo "📊 Generating coverage report..."
    
    if ! command -v cargo-tarpaulin &> /dev/null; then
        echo "⚠️  cargo-tarpaulin is not installed. Installing..."
        cargo install cargo-tarpaulin
    fi
    
    cargo tarpaulin --out Html --output-dir coverage/
    
    echo ""
    echo "✅ Coverage report generated: coverage/tarpaulin-report.html"
    echo "   Open in browser: xdg-open coverage/tarpaulin-report.html 2>/dev/null || open coverage/tarpaulin-report.html"
}

# Main execution
echo "🏃 Starting tests..."
echo ""

START_TIME=$(date +%s)

run_tests

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
echo "⏱️  Tests completed in ${DURATION}s"

if [ "${COVERAGE}" = true ]; then
    run_coverage
fi

echo ""
echo "✨ Test run completed successfully!"