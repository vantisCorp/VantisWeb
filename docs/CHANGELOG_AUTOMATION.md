# Automated Changelog Generation

This document describes the automated changelog generation system implemented for VantisWeb Browser.

## Overview

The changelog is automatically generated from conventional commit messages using `standard-version`. This ensures consistent, readable release notes with minimal manual effort.

## Conventional Commits

All commit messages must follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Commit Types

| Type | Section in Changelog | Description |
|------|---------------------|-------------|
| `feat` | ✨ Features | New features |
| `fix` | 🐛 Bug Fixes | Bug fixes |
| `perf` | ⚡ Performance | Performance improvements |
| `refactor` | ♻️ Refactoring | Code refactoring |
| `docs` | 📚 Documentation | Documentation changes |
| `test` | 🧪 Tests | Adding or updating tests |
| `build` | 📦 Build | Build system changes |
| `ci` | 🔧 CI/CD | CI/CD configuration |
| `chore` | (hidden) | Maintenance tasks |
| `style` | (hidden) | Code style changes |

### Examples

```bash
# New feature
feat(browser): add multi-window workspace support

# Bug fix
fix(tabs): resolve memory leak on tab close

# Breaking change
feat(api)!: change extension API signature

BREAKING CHANGE: The extension API now uses async/await
```

## Usage

### Manual Release

```bash
# Patch release (1.0.0 -> 1.0.1)
npm run release

# Minor release (1.0.0 -> 1.1.0)
npm run release:minor

# Major release (1.0.0 -> 2.0.0)
npm run release:major
```

### Automated Release via GitHub Actions

1. Go to Actions → Changelog Generation
2. Click "Run workflow"
3. Select release type (patch/minor/major)
4. Click "Run workflow"

### Preview Changelog (Dry Run)

```bash
# Preview what the next release would contain
npx standard-version --dry-run
```

## Configuration Files

### .commitlintrc.json
Validates commit messages follow conventional format.

### .versionrc.json
Configures standard-version for changelog generation.

### .husky/commit-msg
Git hook that validates commit messages before allowing commits.

## GitHub Actions Workflow

The `changelog.yml` workflow provides:

1. **Changelog Preview**: Shows changelog preview in PR summaries
2. **Commit Validation**: Validates all PR commits follow conventional format
3. **Automated Release**: Creates releases with generated changelogs
4. **GitHub Release**: Automatically creates GitHub releases with notes

## Changelog Format

Generated changelogs follow this structure:

```markdown
# 📋 VantisWeb Browser - Change Log

## [1.1.0] - 2024-03-07

### ✨ Features
- **browser**: add multi-window workspace support
- **extensions**: implement permission management system

### 🐛 Bug Fixes
- **tabs**: resolve memory leak on tab close
- **navigation**: fix back button history

### ⚡ Performance
- optimize page load time by 40%

### 🧪 Tests
- add 120 new E2E tests for critical flows

### 📚 Documentation
- update API documentation for extensions
```

## Breaking Changes

To indicate a breaking change, use `!` after the type/scope or include a `BREAKING CHANGE:` footer:

```bash
feat(api)!: change extension API signature

# or

feat(api): change extension API signature

BREAKING CHANGE: The extension API now requires async/await
```

## Best Practices

1. **Use imperative mood**: "add feature" not "added feature"
2. **Keep subject lines under 72 characters**
3. **Use scopes to group related changes**: `feat(browser):`, `fix(tabs):`
4. **Reference issues in footer**: `Closes #123`
5. **Write meaningful descriptions** - they become release notes

## Integration with Release Process

1. Developer creates PR with conventional commits
2. GitHub Actions validates commit messages
3. On merge to main, changelog preview is generated
4. When ready for release, maintainer triggers release workflow
5. `standard-version` updates CHANGELOG.md and version numbers
6. GitHub release is created with changelog content

## Troubleshooting

### Commit rejected by hook
Ensure your commit message follows the format:
```
type(scope): description
```

### Changelog not updating
Check that:
1. Commits have correct types
2. You're not using `chore` or `style` (hidden from changelog)
3. There are new commits since last release

### Release not created
Ensure:
1. GITHUB_TOKEN has write permissions
2. You have push access to the repository
3. The workflow is triggered correctly