# Rustic

Rustic is a modular audio synthesis framework written in Rust that provides real-time digital signal processing capabilities for building musical instruments, audio effects, and synthesis applications.

## Frontend

A frontend application is available under the `frontend` folder. This is a simple webgl application that allows to create audio pipelines for the instruments.

It aims to provide a simple user interface to create audio pipelines for the instruments.

## Architecture

The project aims to use the Pipe & Filter architecture, alongside an Event-Driven architecture.

### Pipe & Filter

This architecture is used to create a pipeline of filters that process the audio data. Each filter is a simple function that takes an input and returns an output. The output of a filter is the input of the next filter in the pipeline.
The frontend application aims at providing a simple way to create these pipelines.

### Event-Driven

The event-driven architecture aims at triggering the creation of audio from keyboard events. This is done using the evdev crate, which allows to listen to keyboard events. These events will, depending on the context (provided by the `Application` structure, trigger an instrument to start playing a certain note.

## Development

### Pre-commit Hooks

This project includes pre-commit hooks to ensure code quality. The hooks check code formatting and run tests before allowing commits.

#### Installation

##### Linux/macOS

```bash
./hooks/install.sh
```

##### Windows

```cmd
hooks\install.bat
```

See `hooks/README.md` for more information.

### Documentation

Project documentation is automatically built and published to GitHub Pages when a new release is created. The documentation is generated using `cargo doc` and can be accessed at:
[https://minigrim0.github.io/rustic/](https://minigrim0.github.io/rustic/)
