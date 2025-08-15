# Music generator
This project aims to generate music using rust and the `rodio` crate.

## Frontend
A frontend application is available under the `app` folder. This application is created using the tauri framework, with a Vue.js frontend.

It aims to provide a simple user interface to create audio pipelines for the instruments.

# Architecture
The project aims to use the Pipe & Filter architecture, alongside an Event-Driven architecture.

## Pipe & Filter
This architecture is used to create a pipeline of filters that process the audio data. Each filter is a simple function that takes an input and returns an output. The output of a filter is the input of the next filter in the pipeline.
The frontend application aims at providing a simple way to create these pipelines.

## Event-Driven
The event-dirven architecture aims at triggering the creation of audio from keyboard events. This is done using the evdev crate, which allows to listen to keyboard events. These events will, depending on the context (provided by the `Application` structure, trigger an instrument to start playing a certain note.
