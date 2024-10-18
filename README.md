# Music generator
This project aims to generate music using rust and the `rodio` crate.


# Architecture

## Scoring
The scoring module is responsible for generating the music score. This score can be recorded and played back.
It can either record a complete piece of music in a file for later playback, or record a subset of music for immediate playback.
This sub-scoring module can be used for looping a section of music.

## Music generation
The generation of music is done by the music generation module.


## Inputs
