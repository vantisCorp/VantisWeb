#!/bin/bash

# VantisWeb Browser - Comprehensive Test Runner
# Runs all unit tests, integration tests, and benchmarks

set -e

echo "=========================================="
echo "VantisWeb Browser - Test Suite"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run tests
run_test() {
    local test_name=$1
    local test_command=$2
    
    echo "Running: $test_name"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if eval "$test_command"; then
        echo -e "${GREEN}✓ PASSED${NC}: $test_name"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}✗ FAILED${NC}: $test_name"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    echo ""
}

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Warning: Rust/Cargo not found. Skipping actual test execution.${NC}"
    echo "This script will create a test plan that can be run when Rust is available."
    echo ""
fi

echo "=========================================="
echo "Phase 1: Unit Tests"
echo "=========================================="
echo ""

# Core module unit tests
run_test "Core Module Unit Tests" "cargo test --lib core::"
run_test "Extensions Module Unit Tests" "cargo test --lib extensions::"
run_test "Profiles Module Unit Tests" "cargo test --lib profiles::"
run_test "Web Engine Module Unit Tests" "cargo test --lib engine::"
run_test "UI Module Unit Tests" "cargo test --lib ui::"

echo "=========================================="
echo "Phase 2: Integration Tests"
echo "=========================================="
echo ""

# Integration tests
run_test "Core Integration Tests" "cargo test --test integration_test_core"
run_test "Extensions Integration Tests" "cargo test --test integration_test_extensions"
run_test "Profiles Integration Tests" "cargo test --test integration_test_profiles"
run_test "Web & UI Integration Tests" "cargo test --test integration_test_web_ui"

echo "=========================================="
echo "Phase 3: Performance Benchmarks"
echo "=========================================="
echo ""

# Performance benchmarks
run_test "Core Benchmarks" "cargo bench --bench core_bench"
run_test "Extensions Benchmarks" "cargo bench --bench extensions_bench"
run_test "Profiles Benchmarks" "cargo bench --bench profiles_bench"
run_test "Web Benchmarks" "cargo bench --bench web_bench"

echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo ""
echo "Total Tests: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
echo -e "${RED}Failed: $FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi