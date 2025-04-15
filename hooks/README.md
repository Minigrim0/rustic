# Git Hooks for Rustic Project

This directory contains Git hooks and installation scripts to help maintain code quality.

## Available Hooks

### Pre-Commit Hook

The pre-commit hook performs the following checks before allowing a commit:

1. **Code Formatting**: Runs `cargo fmt --all -- --check` to ensure all code is properly formatted
2. **Tests**: Runs `cargo test` to verify that all tests pass

If any of these checks fail, the commit will be aborted, and you'll need to fix the issues before committing.

## Installation

### Linux/macOS

1. Open a terminal and navigate to the project root directory
2. Run: `./hooks/install.sh`
3. You should see a confirmation that the hooks were installed successfully

### Windows

1. Open Command Prompt or PowerShell and navigate to the project root directory
2. Run: `hooks\install.bat`
3. You should see a confirmation that the hooks were installed successfully

The Windows installation script installs both Bash and PowerShell versions of the pre-commit hook:
- `pre-commit` - Bash script (for Git Bash or WSL users)
- `pre-commit.ps1` - PowerShell script (native Windows support)
- `pre-commit.cmd` - Wrapper script that runs the PowerShell version

On Windows systems, the PowerShell version will be used by default.

## Manual Installation

If the installation scripts don't work for you, you can manually copy the hooks:

1. Copy `hooks/pre-commit` to `.git/hooks/pre-commit`
2. Make sure the hook is executable (on Unix-like systems): `chmod +x .git/hooks/pre-commit`

## Skipping Hooks

If you need to bypass the pre-commit hook in an emergency situation, you can use:

```
git commit --no-verify
```

However, this should be used sparingly, as the hooks are designed to maintain code quality.