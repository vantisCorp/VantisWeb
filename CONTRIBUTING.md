# 🤝 VantisWeb Browser - Contributing Guide

Dziękujemy za zainteresowanie wkładem w rozwój VantisWeb! Ten przewodnik pomoże Ci rozpocząć pracę.

---

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Documentation](#documentation)
- [Submitting Changes](#submitting-changes)
- [Pull Request Process](#pull-request-process)

---

## 📜 Code of Conduct

Jako uczestnicy i opiekunowie zobowiązujemy się do uczynienia każdego udziału w VantisWeb i naszej społeczności doświadczeniem wolnym od nękania dla każdego, niezależnie od poziomu doświadczenia, płci, tożsamości i wyrażenia płciowej, orientacji seksualnej, niepełnosprawności, wyglądu fizycznego, wielkości ciała, rasy, etniczności, wieku, religii lub narodowości.

---

## 🚀 Getting Started

### Prerequisites

- **Rust** 1.75+ [Install Rust](https://www.rust-lang.org/tools/install)
- **Cargo** (comes with Rust)
- **Git** [Install Git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git)
- **Editor**: VS Code, IntelliJ IDEA, or your preferred editor

### System Dependencies

#### Linux (Debian/Ubuntu)
```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.0-dev build-essential curl wget file libssl-dev
```

#### macOS
```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install webkit2gtk
```

#### Windows
- Install [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)
- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)

### Fork and Clone

```bash
# Fork the repository on GitHub
# Clone your fork
git clone https://github.com/YOUR_USERNAME/VantisWeb.git
cd VantisWeb

# Add upstream remote
git remote add upstream https://github.com/vantisCorp/VantisWeb.git
```

### Build the Project

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run the browser
cargo run
```

---

## 🔄 Development Workflow

### 1. Create a Branch

Zawsze twórz nową gałąź dla swoich zmian:

```bash
# Update main branch
git checkout main
git pull upstream main

# Create feature branch
git checkout -b feature/amazing-feature
# or for bugfix
git checkout -b fix/critical-bug
```

**Naming Conventions:**
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test additions/changes
- `chore/` - Maintenance tasks

### 2. Make Your Changes

```bash
# Make your changes
# ... (edit files)

# Format your code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test
```

### 3. Commit Your Changes

```bash
# Stage your changes
git add .

# Commit with descriptive message
git commit -m "feat: Add amazing new feature

This commit adds the amazing new feature that does X and Y.

- Implemented X functionality
- Added Y support
- Updated documentation

Closes #123"
```

**Commit Message Format:**
- Use conventional commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`
- Subject line: max 50 characters
- Body: wrap at 72 characters
- Explain **what** and **why**, not **how**
- Reference issues: `Closes #123`, `Fixes #456`

### 4. Sync and Push

```bash
# Sync with upstream
git fetch upstream
git rebase upstream/main

# Push to your fork
git push origin feature/amazing-feature
```

---

## 📏 Coding Standards

### Rust Guidelines

#### Naming Conventions
```rust
// Functions and variables: snake_case
fn calculate_distance(x: f64, y: f64) -> f64 {
    // ...
}

// Types and structs: PascalCase
struct UserProfile {
    name: String,
    age: u32,
}

// Constants: SCREAMING_SNAKE_CASE
const MAX_CONNECTIONS: usize = 100;

// Modules: snake_case
mod user_management;
```

#### Error Handling
```rust
// Use Result for recoverable errors
fn process_data(data: &str) -> Result<ProcessedData, Error> {
    // ...
}

// Use Option for nullable values
fn find_user(id: u32) -> Option<User> {
    // ...
}

// Never panic in production code
fn safe_divide(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}
```

#### Documentation
```rust
/// Calculates the Euclidean distance between two points.
///
/// # Arguments
///
/// * `x1` - First point's x coordinate
/// * `y1` - First point's y coordinate
/// * `x2` - Second point's x coordinate
/// * `y2` - Second point's y coordinate
///
/// # Returns
///
/// The Euclidean distance as a `f64`
///
/// # Examples
///
/// ```
/// use vantisweb::math::distance;
///
/// let dist = distance(0.0, 0.0, 3.0, 4.0);
/// assert_eq!(dist, 5.0);
/// ```
pub fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
}
```

### Performance Guidelines

#### Memory Management
```rust
// Use String for owned data
let name = String::from("VantisWeb");

// Use &str for borrowed data
fn process_name(name: &str) {
    // ...
}

// Use Vec for owned collections
let items = vec![1, 2, 3];

// Use &[] for borrowed slices
fn sum(items: &[i32]) -> i32 {
    items.iter().sum()
}
```

#### Arc vs Rc
```rust
// Use Arc for thread-safe shared ownership
use std::sync::Arc;

let data = Arc::new(vec![1, 2, 3]);

// Use Rc for single-threaded shared ownership
use std::rc::Rc;

let data = Rc::new(vec![1, 2, 3]);
```

### Code Organization

```
src/
├── core/           # Core systems (kernel, scheduler)
├── security/       # Security modules
├── ui/            # User interface
├── engine/        # Web rendering engine
├── extensions/    # Extension system
├── profiles/      # Profile management
└── utils/         # Utilities
```

---

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run tests in release mode
cargo test --release

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_test_core
```

### Writing Tests

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_calculation() {
        let result = distance(0.0, 0.0, 3.0, 4.0);
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_distance_zero() {
        let result = distance(0.0, 0.0, 0.0, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    #[should_panic]
    fn test_invalid_input() {
        // This should panic
        panic!("Invalid input");
    }
}
```

#### Integration Tests
```rust
// tests/integration_test_profiles.rs
use vantisweb::profiles;

#[test]
fn test_profile_creation() {
    let profile = profiles::create_profile("Test Profile");
    assert_eq!(profile.name(), "Test Profile");
}
```

### Test Coverage

```bash
# Install tarpaulin for coverage
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/
```

---

## 📚 Documentation

### Code Documentation

- **Document all public APIs** with doc comments (`///`)
- **Include examples** for complex functions
- **Explain parameters** and return values
- **Note any panics** that may occur

### README Updates

Update `README.md` when:
- Adding new major features
- Changing build instructions
- Updating dependencies
- Significant API changes

### CHANGELOG

Update `CHANGELOG.md` when:
- Adding new features (Under `Added`)
- Making breaking changes (Under `Changed`)
- Fixing bugs (Under `Fixed`)
- Removing features (Under `Removed`)

---

## 📤 Submitting Changes

### Before Submitting

✅ Run `cargo fmt` to format code
✅ Run `cargo clippy` to lint code
✅ Run `cargo test` to ensure tests pass
✅ Update documentation if needed
✅ Update CHANGELOG.md
✅ Write descriptive commit messages
✅ Ensure your branch is up to date with main

### Create Pull Request

1. Go to your fork on GitHub
2. Click "New Pull Request"
3. Select your branch
4. Fill in the PR template:
   - **Title**: Clear and descriptive
   - **Description**: What and why
   - **Screenshots**: For UI changes
   - **Tests**: Describe what you tested
   - **Breaking Changes**: Note any
   - **Closes**: Reference issues

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Performance improvement

## Testing
Describe how you tested your changes:
- Unit tests: `cargo test`
- Manual testing: ...
- Screenshots: ...

## Checklist
- [ ] My code follows the style guidelines of this project
- [ ] I have performed a self-review of my own code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] I have updated the CHANGELOG.md

## Related Issues
Closes #123, Fixes #456
```

---

## 🔍 Pull Request Process

### Review Process

1. **Automated Checks**: CI/CD will run tests and lints
2. **Code Review**: Maintainers will review your code
3. **Feedback**: Address any comments or suggestions
4. **Approval**: At least one maintainer approval required
5. **Merge**: Maintainer will merge your PR

### What We Look For

✅ **Code Quality**: Clean, readable, well-documented code
✅ **Tests**: Adequate test coverage for new features
✅ **Documentation**: Updated README, API docs, and CHANGELOG
✅ **Performance**: No performance regressions
✅ **Security**: No security vulnerabilities
✅ **Style**: Follows project coding standards
✅ **Breaking Changes**: Clearly documented and justified

### Getting Your PR Merged

1. **Be Patient**: Review may take time
2. **Be Responsive**: Address feedback promptly
3. **Be Open**: Accept constructive criticism
4. **Be Thorough**: Ensure tests pass and docs are updated

---

## 💡 Tips for Good Contributions

### Start Small
- Fix a bug or typo first
- Add a small feature
- Improve documentation
- Add tests

### Ask Questions
- Use GitHub Issues for questions
- Join our Discord server
- Read existing code and documentation
- Don't hesitate to ask for clarification

### Learn from Others
- Review other PRs
- Read the codebase
- Follow the coding standards
- Look at test patterns

---

## 🌟 Becoming a Maintainer

Active contributors who consistently:
- Submit high-quality PRs
- Review other PRs
- Help answer issues
- Improve documentation

...may be invited to become maintainers with commit access.

---

## 📞 Get Help

- **GitHub Issues**: [https://github.com/vantisCorp/VantisWeb/issues](https://github.com/vantisCorp/VantisWeb/issues)
- **Discord**: [https://discord.gg/vantis](https://discord.gg/vantis)
- **Email**: [support@vantis.ai](mailto:support@vantis.ai)

---

## 📜 License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

**Dziękujemy za Twój wkład! 🎉**

[⬆️ Back to Top](README.md)