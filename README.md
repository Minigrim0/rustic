# Rustic

<div align="center">
    <img alt="CI badge" src="https://github.com/minigrim0/rustic/actions/workflows/ci.yml/badge.svg" />
    <img alt="Docs badge" src="https://img.shields.io/badge/docs-latest-blue.svg" />
    <img alt="License badge" src="https://img.shields.io/github/license/minigrim0/rustic" />
    <img alt="Rust version badge" src="https://img.shields.io/badge/rust-1.73%2B-gray" />
</div>

Rustic is a **modular audio synthesis framework** written in Rust that lets you build real‑time DSP pipelines, musical instruments, audio effects, and full‑stack synthesis applications.

## Quick start

```bash
git clone https://github.com/minigrim0/rustic.git
cd rustic
```

Follow the *Getting started* section to create a project, run the frontend, and explore the demo apps.

## Development

### Pre‑commit hooks

The repository uses [pre‑commit](https://pre-commit.com/) to enforce
formatting (rustfmt) and to run the default test suite.

```bash
pre-commit install          # install hooks
pre-commit run --all-files  # test hooks locally
```

### Documentation

Documentation is automatically built & published to GitHub Pages on every push to main.

[https://minigrim0.github.io/rustic/](https://minigrim0.github.io/rustic/)

### Get involved

- Clone the repo:

```bash
git clone https://github.com/minigrim0/rustic.git
cd rustic
```

- Build & run the examples:

```bash
cargo run --example drum_machine
```

- For frontend development:

```bash
cargo tauri dev
```
