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

## Components

The project consists of multile components.

- `rustic/` - The core library, responsible for managing the audio thread and synthesis
- `rustic-meta/` + `rustic-derive/` - Handles deriving metadata for filter structures in the core library
- `rustic-toolkit/` - A frontend using the library for audio analysis, filter graph building and testing (still heavily WIP). Serves as a playground for audio synthesis.
- `frontend/` - Deprecated - Initial attempt at making a frontend for actually playing music, might disappear entirely in the near future.

### Core library architecture

The library implements a real-time-safe audio system using three threads that communicate via lock-free data structures:

#### Thread Responsibilities:

- **Command Thread** (`spawn_command_thread`): Receives Commands from the frontend, validates them, translates to AudioMessage enum, and forwards to render thread. Maintains the App state machine.
- **Render Thread** (`spawn_audio_render_thread`): Owns all Instrument instances, processes AudioMessage queue, calls tick() on instruments, produces audio samples, pushes to ArrayQueue<f32>.
- **CPAL Callback** (`create_cpal_callback`): Real-time audio thread managed by cpal, pulls samples from ArrayQueue<f32>, writes to audio device buffer. Must never block.

```mermaid
graph TB                                                                                                                                                                    
      subgraph "Frontend Process (egui main thread)"                                                                                                                          
          FE_APP["eframe App<br/>─────────<br/>app_sender: Sender&lt;Command&gt;<br/>app_receiver: Receiver&lt;BackendEvent&gt;"]                                             
                                                                                                                                                                              
          subgraph "Tabs"
              TAB_LIVE["LivePlayingTab"]
              TAB_GRAPH["GraphEditorTab<br/>─────────<br/>• Node positions<br/>• Parameter values<br/>• Visual connections<br/>• Palette entries<br/><i>(source of truth for
  UI state)</i>"]
              TAB_SETTINGS["SettingsTab"]
          end

          KEYMAPPER["KeyMapper<br/>─────────<br/>sender: Sender&lt;Command&gt;<br/>device_state: DeviceState"]
      end

      subgraph "Backend (rustic::start_app)"

          subgraph "Command Thread"
              CMD["spawn_command_thread()<br/>─────────<br/><b>Owns:</b><br/>• App (config, rows, run_mode)<br/>• GraphData {<br/>  &nbsp;&nbsp;system: System
  (petgraph),<br/>  &nbsp;&nbsp;filter_map: HashMap&lt;u64, NodeIndex&gt;,<br/>  &nbsp;&nbsp;source_map: HashMap&lt;u64, usize&gt;,<br/>  &nbsp;&nbsp;sink_map:
  HashMap&lt;u64, usize&gt;<br/>}"]
          end

          subgraph "Render Thread"
              RENDER["spawn_audio_render_thread()<br/>─────────<br/><b>Owns:</b><br/>• Vec&lt;Box&lt;dyn Instrument&gt;&gt;<br/>• Option&lt;System&gt; (playing copy)<br/>•
  RenderMode (Instruments / Graph)<br/>• chunk_buffer: Vec&lt;f32&gt;"]
          end

          subgraph "cpal Callback (OS audio thread)"
              CPAL["create_cpal_callback()<br/>─────────<br/><b>Closure captures:</b><br/>• Arc&lt;ArrayQueue&lt;f32&gt;&gt;<br/>• Arc&lt;SharedAudioState&gt;<br/><br/>Copies
   samples → audio device"]
          end
      end

      subgraph "Shared State (Arc, lock-free)"
          SHARED["Arc&lt;SharedAudioState&gt;<br/>─────────<br/>shutdown: AtomicBool<br/>buffer_underruns: AtomicU64<br/>sample_rate: AtomicU32<br/>master_volume: AtomicF32"]
          RINGBUF["Arc&lt;ArrayQueue&lt;f32&gt;&gt;<br/>─────────<br/>Lock-free SPSC ring buffer<br/>size: audio_ring_buffer_size"]
      end

      %% Frontend → Backend channels
      FE_APP -->|"mpsc::channel&lt;Command&gt;<br/>(unbounded)"| CMD
      KEYMAPPER -->|"clone of same Sender&lt;Command&gt;"| CMD
      TAB_LIVE -->|"&Sender&lt;Command&gt;"| FE_APP
      TAB_GRAPH -->|"&Sender&lt;Command&gt;"| FE_APP
      TAB_SETTINGS -->|"&Sender&lt;Command&gt;"| FE_APP

      %% Backend → Frontend channel
      CMD -->|"mpsc::channel&lt;BackendEvent&gt;<br/>(unbounded)<br/>AudioStarted, AudioStopped,<br/>CommandError, GraphError"| FE_APP

      %% Command → Render channel
      CMD -->|"crossbeam::channel&lt;AudioMessage&gt;<br/>(bounded: message_ring_buffer_size)<br/>NoteStart, NoteStop, SetOctave,<br/>SwapGraph*,
  ClearGraph*,<br/>SetRenderMode*, GraphSetParameter*"| RENDER

      %% Render → cpal ring buffer
      RENDER -->|"push samples"| RINGBUF
      RINGBUF -->|"pop samples"| CPAL

      %% Shared state access
      SHARED -.->|"read shutdown flag"| RENDER
      SHARED -.->|"read shutdown flag,<br/>write shutdown=true"| CMD
      SHARED -.->|"increment buffer_underruns"| CPAL

      %% Graph duplication flow (annotated)
      CMD -. "On Play: clone System<br/>→ SwapGraph(Box&lt;System&gt;)<br/><br/>On Stop: ClearGraph<br/>(render drops its copy)" .-> RENDER

      CPAL -->|"audio device output"| SPEAKER["Speaker"]

      style SHARED fill:#2d2d3d,stroke:#7a7aff,color:#fff
      style RINGBUF fill:#2d2d3d,stroke:#7a7aff,color:#fff
      style CMD fill:#1a3a1a,stroke:#4a4,color:#fff
      style RENDER fill:#3a1a1a,stroke:#a44,color:#fff
      style CPAL fill:#3a2a1a,stroke:#a84,color:#fff
      style TAB_GRAPH fill:#1a2a3a,stroke:#48a,color:#fff
```

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
