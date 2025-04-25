# GitHub Actions Workflows

This directory contains GitHub Actions workflows for automating various tasks in the Rustic project.

## Available Workflows

### Publish Documentation

**File:** `publish-docs.yml`

This workflow automatically publishes the Rust documentation to GitHub Pages whenever a new release is created.

#### How it works

1. When a new release is created on GitHub, this workflow is triggered
2. It checks out the repository code
3. Sets up Rust with the stable toolchain
4. Generates documentation using `cargo doc`
5. Deploys the generated documentation to the `gh-pages` branch

#### Requirements

For this workflow to function properly:

1. The repository needs to have GitHub Pages enabled and configured to use the `gh-pages` branch
2. The workflow needs proper permissions to push to the repository

#### Enabling GitHub Pages

To enable GitHub Pages for your documentation:

1. Go to your repository's Settings
2. Navigate to the "Pages" section
3. Under "Source", select the `gh-pages` branch
4. Click "Save"

After the workflow runs for the first time and the `gh-pages` branch is created, your documentation will be available at:
`https://[username].github.io/rustic/`

#### Customization

You can customize this workflow by editing the `publish-docs.yml` file:

- Change when the workflow triggers (e.g., on push to main instead of on release)
- Modify the documentation generation command to include dependencies or set other options
- Adjust the GitHub Pages deployment configuration