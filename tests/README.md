# VantisWeb Browser Test Suite

This directory contains comprehensive tests for the VantisWeb Browser project.

## Test Structure

```
tests/
├── auth/              # Authentication and security tests
│   ├── totp_test.rs          # TOTP implementation tests
│   └── session_test.rs       # Session management tests
├── core/              # Core browser engine tests
│   ├── kernel_test.rs        # Vantis Kernel tests
│   ├── scheduler_test.rs     # Task scheduler tests
│   └── storage_test.rs       # Storage system tests
├── profiles/          # Profile management tests
│   └── profile_manager_test.rs  # Profile manager tests
├── sync/              # Cloud sync tests
│   └── providers_test.rs     # Sync provider tests
├── integration/       # Integration tests
│   └── browser_workflow_test.rs  # End-to-end workflow tests
└── *_tests.rs         # Legacy test files
```

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Module
```bash
cargo test --test kernel_test
```

### Run Specific Test
```bash
cargo test test_kernel_initialization
```

### Run Tests with Output
```bash
cargo test -- --nocapture
```

### Run Tests with Coverage
```bash
cargo tarpaulin --out Html
```

## Test Categories

### Unit Tests
- Test individual functions and modules
- Fast to run
- No external dependencies
- Located in `tests/` subdirectories

### Integration Tests
- Test interactions between modules
- Test complete workflows
- May require external resources
- Located in `tests/integration/`

## Coverage Goals

- Target coverage: **80%**
- Critical modules: **90%+**
- Current coverage: TBD

## Writing Tests

1. Place test files in appropriate subdirectory
2. Use descriptive test names: `test_<functionality>_<scenario>`
3. Follow the Arrange-Act-Assert pattern
4. Keep tests focused and independent
5. Add documentation for complex tests

## CI/CD Integration

Tests run automatically on:
- Pull requests
- Main branch commits
- Scheduled runs (nightly)

## Contributing

When adding new features, include corresponding tests:
1. Unit tests for new functions
2. Integration tests for new workflows
3. Update this README with test descriptions