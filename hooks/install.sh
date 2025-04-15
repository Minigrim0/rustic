#!/bin/bash

# Linux/macOS installation script for git hooks

HOOKS_DIR="$(git rev-parse --git-dir)/hooks"
SCRIPTS_DIR="$(dirname "$0")"

echo "Installing git hooks..."

# Copy the pre-commit hook
cp "$SCRIPTS_DIR/pre-commit" "$HOOKS_DIR/pre-commit"

# Make it executable
chmod +x "$HOOKS_DIR/pre-commit"

echo "Hooks installed successfully!"
echo "Pre-commit hook will now run before each commit to check formatting and run tests."