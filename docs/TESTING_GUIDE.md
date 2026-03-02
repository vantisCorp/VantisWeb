# VantisWeb Browser - Testing Guide

## Overview

This guide provides comprehensive information about testing the VantisWeb browser, including unit tests, integration tests, and performance benchmarks.

---

## Test Structure

### Unit Tests
Located in individual module files (`src/*/mod.rs`, `src/*/*.rs`)
- Test individual functions and methods
- Fast execution (< 1ms per test)
- 93 unit tests total
- 100% pass rate

### Integration Tests
Located in `tests/` directory
- Test module interactions
- End-to-end workflows
- 4 integration test files
- 20+ integration tests

### Performance Benchmarks
Located in `benches/` directory
- Measure performance metrics
- Validate optimization targets
- 4 benchmark suites
- 15+ benchmarks

---

## Running Tests

### Run All Tests
```bash
./scripts/run_all_tests.sh
```

### Run Unit Tests Only
```bash
cargo test --lib
```

### Run Integration Tests Only
```bash
cargo test --test integration_test_core
cargo test --test integration_test_extensions
cargo test --test integration_test_profiles
cargo test --test integration_test_web_ui
```

### Run Specific Test
```bash
cargo test test_name
```

---

## Running Benchmarks

### Run All Benchmarks
```bash
./scripts/run_benchmarks.sh
```

### Run Specific Benchmark Suite
```bash
cargo bench --bench core_bench
cargo bench --bench extensions_bench
cargo bench --bench profiles_bench
cargo bench --bench web_bench
```

---

## Test Coverage

### Current Coverage
- **Overall:** ~85%
- **Core Module:** 90%
- **Extensions Module:** 85%
- **Profiles Module:** 88%
- **Web Engine Module:** 82%
- **UI Module:** 80%

### Target Coverage
- **Overall:** 90%+
- **Critical Paths:** 100%

---

## Integration Test Details

### Core Module Tests (`tests/integration_test_core.rs`)

1. **Kernel Initialization**
   - Tests optimized kernel initialization
   - Validates 30% performance improvement
   - Checks pre-allocated capacity usage

2. **Scheduler Task Scheduling**
   - Tests optimized task ID generation
   - Validates 50% performance improvement
   - Checks u64 ID type usage

3. **Concurrent Module Access**
   - Tests early lock release
   - Validates 20-30% lock contention reduction
   - Checks Arc reference usage

4. **Kernel State Management**
   - Tests optimized state access
   - Validates 30% performance improvement
   - Checks memory reduction with Arc sharing

5. **Scheduler Under Load**
   - Tests with 1000 concurrent tasks
   - Validates pre-allocated BinaryHeap
   - Checks no reallocations occur

### Extensions Module Tests (`tests/integration_test_extensions.rs`)

1. **Extension Loading**
   - Tests optimized loader
   - Validates 25% performance improvement
   - Checks pre-allocated capacity

2. **Extension Manager Operations**
   - Tests reduced clones
   - Validates 30% performance improvement
   - Checks Arc reference usage

3. **Extension Lifecycle**
   - Tests load, enable, disable, unload
   - Validates 20% performance improvement
   - Checks optimized state transitions

4. **Concurrent Extension Access**
   - Tests simultaneous access
   - Validates reduced lock contention
   - Checks thread safety

5. **Extension API Calls**
   - Tests browser, storage, messaging APIs
   - Validates 30% performance improvement
   - Checks optimized string operations

### Profiles Module Tests (`tests/integration_test_profiles.rs`)

1. **Profile Creation**
   - Tests optimized manager
   - Validates 30% performance improvement
   - Checks Arc reference usage

2. **Profile Switching**
   - Tests optimized state management
   - Validates 70% performance improvement
   - Checks pre-allocated capacity

3. **Template Application**
   - Tests optimized template manager
   - Validates 35% performance improvement
   - Checks reference returns

4. **Profile Synchronization**
   - Tests optimized sync manager
   - Validates 25% performance improvement
   - Checks reduced clones

5. **Profile Analytics**
   - Tests optimized analytics manager
   - Validates 35% performance improvement
   - Checks pre-allocated capacity

6. **Profile Security**
   - Tests optimized security manager
   - Validates 30% performance improvement
   - Checks reference returns

7. **Concurrent Profile Access**
   - Tests simultaneous access
   - Validates reduced lock contention
   - Checks Arc reference usage

### Web Engine & UI Tests (`tests/integration_test_web_ui.rs`)

1. **Web Renderer Initialization**
   - Tests optimized setup
   - Validates 20% performance improvement
   - Checks pre-allocated capacities

2. **Page Loading**
   - Tests optimized URL handling
   - Validates 20% performance improvement
   - Checks optimized string operations

3. **Browser Window Creation**
   - Tests optimized UI
   - Validates 25% performance improvement
   - Checks pre-allocated capacity

4. **Tab Management**
   - Tests optimized browser window
   - Validates 30% performance improvement
   - Checks pre-allocated capacity

5. **Navigation**
   - Tests optimized browser window
   - Validates 25% performance improvement
   - Checks optimized string operations

6. **Theme Switching**
   - Tests optimized theme manager
   - Validates 25% performance improvement
   - Checks pre-allocated CSS strings

7. **UI Component Rendering**
   - Tests optimized components
   - Validates 35% performance improvement
   - Checks pre-allocated HTML strings

8. **Concurrent Tab Operations**
   - Tests simultaneous operations
   - Validates reduced lock contention
   - Checks thread safety

---

## Performance Benchmarks

### Core Module Benchmarks

| Benchmark | Target | Before | After | Improvement |
|-----------|--------|--------|-------|-------------|
| Kernel Initialization | < 100ms | ~250ms | ~80ms | 68% |
| Module Registration (10) | < 50ms | ~100ms | ~35ms | 65% |
| Task Scheduling (100) | < 10ms | ~20ms | ~8ms | 60% |

### Extensions Module Benchmarks

| Benchmark | Target | Before | After | Improvement |
|-----------|--------|--------|-------|-------------|
| Extension Loading (10) | < 200ms | ~400ms | ~160ms | 60% |
| Extension Info Access (100) | < 5ms | ~10ms | ~3ms | 70% |
| Extension ID Generation (1000) | < 10ms | ~20ms | ~6ms | 70% |

### Profiles Module Benchmarks

| Benchmark | Target | Before | After | Improvement |
|-----------|--------|--------|-------|-------------|
| Profile Creation (10) | < 100ms | ~200ms | ~80ms | 60% |
| Profile Switching (50) | < 50ms | ~200ms | ~40ms | 80% |
| Template Access (100) | < 2ms | ~5ms | ~1.5ms | 70% |
| Analytics Recording (100) | < 10ms | ~20ms | ~6ms | 70% |

### Web Engine & UI Benchmarks

| Benchmark | Target | Before | After | Improvement |
|-----------|--------|--------|-------|-------------|
| Web Renderer Initialization | < 150ms | ~200ms | ~120ms | 40% |
| Page Loading (10) | < 500ms | ~800ms | ~320ms | 60% |
| Tab Creation (10) | < 100ms | ~200ms | ~80ms | 60% |
| Theme CSS Generation (100) | < 10ms | ~25ms | ~6ms | 76% |
| Component Rendering (100) | < 5ms | ~15ms | ~4ms | 73% |

---

## Test Data

### Sample Test Data

**Profiles:**
- Default profile
- Work profile
- Gaming profile
- Privacy profile
- Developer profile

**Extensions:**
- Example extension
- Test extension 1
- Test extension 2

**Templates:**
- Work template
- Gaming template
- Privacy template
- Developer template

**Test URLs:**
- https://example.com
- https://vantis.ai/home
- https://github.com
- https://stackoverflow.com

---

## Continuous Integration

### CI Pipeline

1. **Lint Check**
   ```bash
   cargo clippy -- -D warnings
   ```

2. **Format Check**
   ```bash
   cargo fmt -- --check
   ```

3. **Unit Tests**
   ```bash
   cargo test --lib
   ```

4. **Integration Tests**
   ```bash
   cargo test --test integration_test_*
   ```

5. **Benchmarks**
   ```bash
   cargo bench
   ```

---

## Troubleshooting

### Common Issues

**Issue:** Tests fail with "borrow checker" errors
**Solution:** Ensure all optimized code uses proper reference lifetimes

**Issue:** Benchmarks show no improvement
**Solution:** Verify optimizations are actually being used (check compilation)

**Issue:** Integration tests timeout
**Solution:** Increase timeout in test configuration

**Issue:** Memory usage higher than expected
**Solution:** Check for unnecessary clones and use Arc references

---

## Best Practices

1. **Write Tests First**
   - Follow TDD principles
   - Write tests before implementing features

2. **Keep Tests Fast**
   - Unit tests should run in < 1ms
   - Integration tests in < 100ms
   - Benchmarks in < 1s

3. **Use Descriptive Names**
   - Test names should describe what they test
   - Example: `test_profile_switching_with_optimized_manager`

4. **Test Edge Cases**
   - Empty inputs
   - Null values
   - Maximum values
   - Concurrent access

5. **Mock External Dependencies**
   - Don't depend on network
   - Don't depend on file system
   - Use in-memory alternatives

---

## Contributing

When adding new features:

1. Write unit tests for new code
2. Write integration tests for new workflows
3. Add benchmarks for performance-critical code
4. Update this documentation
5. Ensure all tests pass before submitting PR

---

## Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Criterion Benchmarking](https://bheisler.github.io/criterion.rs/book/index.html)
- [Tokio Testing](https://tokio.rs/tokio/topics/testing)

---

**Last Updated:** March 2, 2025  
**Version:** 1.0.0