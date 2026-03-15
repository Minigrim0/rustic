# Rustic Commands Reference

This document provides a comprehensive overview of all commands in the Rustic project, organized by category with their parameters, thread handling locations, and corresponding AudioMessages.

---

## Thread Architecture

Rustic uses a three-thread architecture for command processing and audio rendering:

1. **Command Thread** (`command-processor`) - Receives commands from frontend, validates them, updates app state, and forwards audio messages
2. **Audio Render Thread** (`audio-render`) - Generates audio samples by processing instrument/graph state
3. **Main Thread** (implicit) - CPAL audio callback thread that consumes samples from ring buffer

### Command Flow

```
Frontend/User → Command Thread → Audio Render Thread → CPAL Callback → Audio Output
                      ↓
                  App State
```

### Thread Handling Legend

| Symbol | Meaning |
|--------|---------|
| **CT** | **Command Thread** - Handled in command thread only (app state update) |
| **CT→RT** | **Command Thread → Render Thread** - Translated to `AudioMessage` and forwarded |
| **RT** | **Render Thread** - Processed directly in audio render thread |
| **CT (Graph)** | **Command Thread (Graph)** - Special graph handling in command thread |

---

## Table of Contents

- [System Commands](#system-commands)
- [Settings Commands](#settings-commands)
- [Live Performance Commands](#live-performance-commands)
- [Loop Commands](#loop-commands)
- [Mix Commands](#mix-commands)
- [Performance Commands](#performance-commands)
- [Effect Commands](#effect-commands)
- [Graph Commands](#graph-commands)
- [AudioMessages Without Commands](#audiomessages-without-commands)
- [Tauri Commands (Toolkit)](#tauri-commands-toolkit)

---

## System Commands

Basic system control commands.

| Command | Parameters | Thread Handling | AudioMessage | Description |
|---------|-----------|----------------|--------------|-------------|
| `Quit` | None | **CT→RT** | `AudioMessage::Shutdown` | Exit application, shutdown all threads |
| `Reset` | None | **CT** | *(none)* | Reset application state |

**Processing Flow:**
- `Quit`: Command thread → Sets shutdown flag → Sends `AudioMessage::Shutdown` → Render thread exits
- `Reset`: Command thread only → Updates app state

**Implementation:** `rustic/src/app/commands/system.rs`
**Handler:** `rustic/src/audio/command_thread.rs:233`

---

## Settings Commands

Application settings and configuration management.

| Command | Parameters | Thread Handling | AudioMessage | Description |
|---------|-----------|----------------|--------------|-------------|
| `SwitchKeyboardLayout` | `layout: String` | **CT** | *(none)* | Change keyboard layout |
| `ToggleHelp` | None | **CT** | *(none)* | Show/hide help |
| `Undo` | None | **CT** | *(none)* | Undo last action |
| `Redo` | None | **CT** | *(none)* | Redo last undone action |
| `TakeSnapshot` | None | **CT** | *(none)* | Save current state snapshot |
| `RestoreSnapshot` | `index: usize` | **CT** | *(none)* | Restore a specific snapshot |
| `LinkAll` | None | **CT** | *(none)* | Link all rows together |
| `UnlinkAll` | None | **CT** | *(none)* | Unlink all rows |
| `SwapRows` | None | **CT** | *(none)* | Swap row positions |
| `CopyRowSettings` | `from: u8, to: u8` | **CT** | *(none)* | Copy settings between rows |
| `ToggleMetronome` | None | **CT** | *(none)* | Enable/disable metronome |
| `SetTempo` | `bpm: u32` | **CT** | *(none)* | Set tempo in BPM |
| `TempoUp` | None | **CT** | *(none)* | Increase tempo |
| `TempoDown` | None | **CT** | *(none)* | Decrease tempo |
| `StartSessionRecording` | None | **CT** | *(none)* | Begin session recording |
| `StopSessionRecording` | None | **CT** | *(none)* | End session recording |
| `PlaySession` | None | **CT** | *(none)* | Play recorded session |
| `StopSession` | None | **CT** | *(none)* | Stop session playback |
| `SaveSession` | `path: String` | **CT** | *(none)* | Save session to file |
| `LoadSession` | `path: String` | **CT** | *(none)* | Load session from file |
| `ListOutputDevices` | None | **CT** | *(none)* | List available output devices |
| `SelectOutputDevice` | `device: String` | **CT** | *(none)* | Select audio output device |

**Processing Flow:**
- All settings commands: Command thread → `app.on_event()` → App state update
- No audio thread interaction (no translation to `AudioMessage`)

**Implementation:** `rustic/src/app/commands/settings.rs`
**Handler:** `rustic/src/audio/command_thread.rs:256` → `rustic/src/app/app.rs:166`

---

## Live Performance Commands

Real-time performance and note control.

| Command | Parameters | Thread Handling | AudioMessage | Description |
|---------|-----------|----------------|--------------|-------------|
| `NoteStart` | `note: u8, row: u8, velocity: f32` | **CT→RT** | `InstrumentAudioMessage::NoteStart { instrument_idx, note, velocity }` | Trigger note on (velocity: 0.0-1.0) |
| `NoteStop` | `note: u8, row: u8` | **CT→RT** | `InstrumentAudioMessage::NoteStop { instrument_idx, note }` | Release note |
| `OctaveUp` | `row: u8` | **CT→RT** | `InstrumentAudioMessage::SetOctave { row, octave }` | Increase octave for row |
| `OctaveDown` | `row: u8` | **CT→RT** | `InstrumentAudioMessage::SetOctave { row, octave }` | Decrease octave for row |
| `SetOctave` | `octave: u8, row: u8` | **CT→RT** | `InstrumentAudioMessage::SetOctave { row, octave }` | Set specific octave (0-8) |
| `LinkOctaves` | None | **CT** | *(none)* | Link octave controls across rows |
| `UnlinkOctaves` | None | **CT** | *(none)* | Unlink octave controls |
| `SelectInstrument` | `index: usize, row: u8` | **CT** | *(none)* | Select instrument for row |
| `NextInstrument` | `row: u8` | **CT** | *(none)* | Switch to next instrument |
| `PreviousInstrument` | `row: u8` | **CT** | *(none)* | Switch to previous instrument |
| `LinkInstruments` | None | **CT** | *(none)* | Link instrument selection |
| `UnlinkInstruments` | None | **CT** | *(none)* | Unlink instrument selection |

**Processing Flow:**
- **CT→RT commands:**
  1. Command thread → Validation (`live.rs:22`)
  2. App state update (`app.on_event()`)
  3. Translation to `InstrumentAudioMessage` (`mod.rs:56-86`)
  4. Send to render thread
  5. Render thread → Process message (`render_thread.rs:101`)
- **CT only commands:** Command thread → App state update only

**Implementation:** `rustic/src/app/commands/live.rs`
**Validation:** `rustic/src/app/commands/live.rs:22` (row bounds, velocity/octave ranges)
**Translation:** `rustic/src/app/commands/mod.rs:56`
**Render Handler:** `rustic/src/audio/render_thread.rs:101`

---

## Loop Commands

Loop recording and playback control.

| Command | Parameters | Thread Handling | AudioMessage | Description |
|---------|-----------|----------------|--------------|-------------|
| `StartRecording` | None | **CT** | *(none - TODO)* | Begin loop recording |
| `StopRecording` | None | **CT** | *(none - TODO)* | End loop recording |
| `PlayLoop` | None | **CT** | *(none - TODO)* | Start loop playback |
| `StopLoop` | None | **CT** | *(none - TODO)* | Stop loop playback |
| `ClearLoop` | None | **CT** | *(none - TODO)* | Clear current loop |
| `LoopRepeat` | `enabled: bool` | **CT** | *(none - TODO)* | Enable/disable loop repeat |
| `LoopRepeatCount` | `count: u32` | **CT** | *(none - TODO)* | Set number of loop repetitions |
| `SaveLoopToSlot` | `slot: u8, row: u8` | **CT** | *(none - TODO)* | Save loop to memory slot |
| `LoadLoopFromSlot` | `slot: u8, row: u8` | **CT** | *(none - TODO)* | Load loop from memory slot |
| `ClearLoopSlot` | `slot: u8` | **CT** | *(none - TODO)* | Clear a loop slot |
| `ToggleLoopSlots` | `slot_a: u8, slot_b: u8` | **CT** | *(none - TODO)* | Toggle between two loop slots |

**Processing Flow:**
- Command thread only → App state management
- No current translation to audio messages (future work)

**Implementation:** `rustic/src/app/commands/looping.rs`
**Handler:** `rustic/src/audio/command_thread.rs:244`

---

## Mix Commands

Audio mixing and volume control.

| Command | Parameters | Thread Handling | AudioMessage | Description |
|---------|-----------|----------------|--------------|-------------|
| `VolumeUp` | `row: u8` | **CT** | *(none - state-based)* | Increase volume for row |
| `VolumeDown` | `row: u8` | **CT** | *(none - state-based)* | Decrease volume for row |
| `SetVolume` | `level: f32, row: u8` | **CT** | *(none - state-based)* | Set volume level |
| `Mute` | `row: u8` | **CT** | *(none - state-based)* | Mute/unmute specific row |
| `MuteAll` | None | **CT** | *(none - state-based)* | Mute all rows |

**Processing Flow:**
- Command thread → App state update
- Volume applied during audio generation in render thread (reads state)
- No explicit audio message (state-based rendering)

**Implementation:** `rustic/src/app/commands/mix.rs`
**Handler:** `rustic/src/audio/command_thread.rs:244`

---

## Performance Commands

Real-time performance effects and modulation.

| Command | Parameters | Thread Handling | AudioMessage | Description |
|---------|-----------|----------------|--------------|-------------|
| `PitchBendUp` | `amount: f32, row: u8` | **CT** | *(none - state-based)* | Bend pitch up |
| `PitchBendDown` | `amount: f32, row: u8` | **CT** | *(none - state-based)* | Bend pitch down |
| `PitchBendReset` | `row: u8` | **CT** | *(none - state-based)* | Reset pitch bend to center |
| `Vibrato` | `depth: f32, row: u8` | **CT** | *(none - state-based)* | Apply vibrato effect |
| `Tremolo` | `depth: f32, row: u8` | **CT** | *(none - state-based)* | Apply tremolo effect |

**Processing Flow:**
- Command thread → App state update
- Effects applied during instrument tick in render thread

**Implementation:** `rustic/src/app/commands/performance.rs`
**Handler:** `rustic/src/audio/command_thread.rs:244`

---

## Effect Commands

Audio effects processing.

| Command | Parameters | Thread Handling | AudioMessage | Description |
|---------|-----------|----------------|--------------|-------------|
| `Reverb` | `amount: f32, row: u8` | **CT** | *(none - state-based)* | Apply reverb effect |
| `Delay` | `time: f32, feedback: f32, row: u8` | **CT** | *(none - state-based)* | Apply delay effect |
| `Chorus` | `depth: f32, row: u8` | **CT** | *(none - state-based)* | Apply chorus effect |
| `Filter` | `frequency: f32, resonance: f32, row: u8` | **CT** | *(none - state-based)* | Apply filter |
| `ToggleDistortion` | `row: u8` | **CT** | *(none - state-based)* | Enable/disable distortion |

**Processing Flow:**
- Command thread → App state update
- Effects applied during instrument processing in render thread

**Implementation:** `rustic/src/app/commands/effect.rs`
**Handler:** `rustic/src/audio/command_thread.rs:244`

---

## Graph Commands

Audio graph node management and routing.

| Command | Parameters | Thread Handling | AudioMessage | Description |
|---------|-----------|----------------|--------------|-------------|
| `AddNode` | `node_type: String, kind: NodeKind, position: (f32, f32)` | **CT (Graph)** | *(none)* | Add node to graph |
| `RemoveNode` | `id: u64` | **CT (Graph)** | *(none)* | Remove node from graph |
| `Connect` | `from: u64, from_port: usize, to: u64, to_port: usize` | **CT (Graph)** | *(none)* | Connect two nodes |
| `Disconnect` | `from: u64, to: u64` | **CT (Graph)** | *(none)* | Disconnect two nodes |
| `SetParameter` | `node_id: u64, param_name: String, value: f32` | **CT (Graph)→RT** | `GraphAudioMessage::SetParameter { node_index, param_name, value }` | Update node parameter |
| `SetNodePosition` | `id: u64, position: (f32, f32)` | **CT** | *(none)* | Update node position (UI only) |
| `Play` | None | **CT (Graph)→RT** | `AudioMessage::SetRenderMode(Graph)` | Start graph playback |
| `Pause` | None | **CT→RT** | `AudioMessage::SetRenderMode(Instruments)` + `GraphAudioMessage::Clear` | Pause graph playback |
| `Stop` | None | **CT→RT** | `AudioMessage::SetRenderMode(Instruments)` + `GraphAudioMessage::Clear` | Stop graph playback |
| `SaveGraph` | `path: String` | **CT** | *(none - frontend)* | Save graph to file (frontend) |
| `LoadGraph` | `path: String` | **CT** | *(none - frontend)* | Load graph from file (frontend) |

**Processing Flow:**
- **Graph Management (CT):**
  1. Command thread maintains `GraphData` structure
  2. Builds audio graph (`System`) with petgraph
  3. Manages node/connection mappings
- **Parameter Updates (CT→RT):**
  1. Command thread sends `GraphAudioMessage::SetParameter`
  2. Render thread applies to live graph (`render_thread.rs:141`)
- **Playback Control (CT→RT):**
  1. `Play`: Compute topology → Send `SetRenderMode(Graph)`
  2. `Stop/Pause`: Send `SetRenderMode(Instruments)` + `Clear`

**Implementation:** `rustic/src/app/commands/graph.rs`
**Graph Handler:** `rustic/src/audio/command_thread.rs:83-206`
**Render Handler:** `rustic/src/audio/render_thread.rs:133`

**Node Kinds:**
- `Generator`: Audio source nodes (oscillators, samplers)
- `Filter`: Audio processing nodes (filters, effects)
- `Sink`: Audio output nodes (speakers, recording)

---

## AudioMessages Without Commands

These AudioMessages can be sent directly to the render thread but have no corresponding high-level Command. They are used internally or for future features.

| AudioMessage | Parameters | Thread | Description | Status |
|--------------|-----------|--------|-------------|--------|
| `AudioMessage::SetMasterVolume` | `volume: f32` | **RT** | Set global master volume | ⚠️ No command exists |
| `AudioMessage::SetSampleRate` | `rate: u32` | **RT** | Change sample rate (requires audio restart) | ⚠️ No command exists |
| `GraphAudioMessage::Swap` | `System` | **RT** | Replace entire audio graph system | ⚠️ Not yet used (TODO at line 186) |

**Notes:**
- `SetMasterVolume`: Could be added as a `MixCommand` in the future
- `SetSampleRate`: Typically set at initialization, changing requires audio system restart
- `GraphAudioMessage::Swap`: Intended for swapping the entire graph when `Play` is called, but currently not implemented (needs `System` to be `Clone`)

**Processing:**
- These messages bypass the command validation/translation pipeline
- Processed directly in render thread via `process_audio_message()`
- Currently handled with no-op or warnings in some cases

**Implementation:** `rustic/src/audio/messages.rs`
**Handler:** `rustic/src/audio/render_thread.rs:165`

---

## Tauri Commands (Toolkit)

Frontend-accessible commands for the Rustic Toolkit application. These run in the Tauri backend thread and are **not** part of the main Command/AudioMessage system.

### Metadata Commands

| Command | Parameters | Thread Handling | Description |
|---------|-----------|----------------|-------------|
| `get_graph_metadata` | None | **Tauri Thread** | Get available node types (generators, filters, sinks) |

**Processing:** Synchronous function call, returns metadata immediately
**Implementation:** `rustic-toolkit/src-tauri/src/commands/meta.rs:6`

---

### Audio Analysis Commands

| Command | Parameters | Thread Handling | Description |
|---------|-----------|----------------|-------------|
| `analyze_audio_file` | `path: String, state: State<RwLock<AudioState>>` | **Tauri Thread** | Load and analyze audio file |

**Processing Flow:**
1. Tauri thread acquires write lock on `AudioState`
2. Loads audio file into buffer
3. Computes global FFT, RMS, pitch
4. Caches buffer for windowed queries
5. Returns `AudioSummary`

**Implementation:** `rustic-toolkit/src-tauri/src/commands/analyze.rs:16`

---

### Audio Query Commands

| Command | Parameters | Thread Handling | Description |
|---------|-----------|----------------|-------------|
| `get_waveform` | `start: f64, end: f64, target_points: u32, state` | **Tauri Thread** | Get downsampled waveform data |
| `get_spectrum` | `start: f64, end: f64, top_count: usize, min_peak_distance: f32, state` | **Tauri Thread** | Get FFT spectrum for time window |
| `get_top_frequencies` | `start: f64, end: f64, freq_lo: f32, freq_hi: f32, top_count: usize, min_peak_distance: f32, state` | **Tauri Thread** | Get top frequency peaks in range |
| `get_spectrogram` | `start: f64, end: f64, state` | **Tauri Thread** | Get spectrogram (STFT) data |

**Processing Flow:**
1. Tauri thread acquires read lock on `AudioState`
2. Extracts sample slice for time range
3. Performs analysis (FFT/STFT/downsampling)
4. Returns data structure for frontend rendering

**Implementation:** `rustic-toolkit/src-tauri/src/commands/query.rs`

---

### Utility Commands

| Command | Parameters | Thread Handling | Description |
|---------|-----------|----------------|-------------|
| `frequency_to_note_command` | `frequency: f32` | **Tauri Thread** | Convert Hz to note name |
| `save_analysis` | `path: String, summary: AudioSummary` | **Tauri Thread** | Save analysis to JSON file |

**Implementation:** `rustic-toolkit/src-tauri/src/commands/utils.rs`

---

### Rustic Control Commands

| Command | Parameters | Thread Handling | Description |
|---------|-----------|----------------|-------------|
| `change_render_mode` | `render_mode: String, rustic_state: State<Mutex<RusticState>>` | **Tauri Thread** | Change rendering mode (incomplete) |

**Status:** ⚠️ Incomplete implementation
**Implementation:** `rustic-toolkit/src-tauri/src/commands/rustic.rs:10`

---

## Command Processing Details

### Command Thread (`rustic/src/audio/command_thread.rs`)

**Responsibilities:**
1. Receive commands from frontend via `Receiver<Command>`
2. Validate commands against app state
3. Update app state via `app.on_event()`
4. Translate commands to `AudioMessage` via `translate_to_audio_message()`
5. Send audio messages to render thread via `crossbeam::channel::Sender<AudioMessage>`
6. Manage graph system (`GraphData` struct)
7. Send events/errors back to frontend via `Sender<BackendEvent>`

**Key Functions:**
- `spawn_command_thread()` - Thread entry point (line 217)
- `handle_graph_command()` - Graph command processing (line 83)

---

### Audio Render Thread (`rustic/src/audio/render_thread.rs`)

**Responsibilities:**
1. Receive audio messages from command thread
2. Process `InstrumentAudioMessage` (note on/off, octave)
3. Process `GraphAudioMessage` (parameter updates, graph swap)
4. Generate audio samples in chunks (configurable size)
5. Write samples to lock-free ring buffer (`ArrayQueue`)
6. Switch between `RenderMode::Instruments` and `RenderMode::Graph`

**Key Functions:**
- `spawn_audio_render_thread()` - Thread entry point (line 23)
- `process_audio_message()` - Message dispatcher (line 165)
- `process_instrument_message()` - Instrument control (line 101)
- `process_graph_message()` - Graph control (line 133)

---

### Audio Messages (`rustic/src/audio/messages.rs`)

**Complete Message Type Hierarchy:**

```rust
AudioMessage
├── Instrument(InstrumentAudioMessage)
│   ├── NoteStart { instrument_idx, note, velocity }       [from: Live::NoteStart]
│   ├── NoteStop { instrument_idx, note }                  [from: Live::NoteStop]
│   └── SetOctave { row, octave }                          [from: Live::OctaveUp/Down/SetOctave]
│
├── Graph(GraphAudioMessage)
│   ├── SetParameter { node_index, param_name, value }     [from: Graph::SetParameter]
│   ├── Swap(System)                                       [NO COMMAND - TODO]
│   └── Clear                                              [from: Graph::Pause/Stop]
│
├── SetRenderMode(RenderMode)
│   ├── Instruments                                        [from: Graph::Pause/Stop]
│   └── Graph                                              [from: Graph::Play]
│
├── SetMasterVolume { volume }                             [NO COMMAND]
├── SetSampleRate { rate }                                 [NO COMMAND]
└── Shutdown                                               [from: System::Quit]
```

**Legend:**
- `[from: ...]` - Generated by this Command
- `[NO COMMAND]` - No corresponding Command exists
- `[TODO]` - Planned but not implemented

---

## Error Handling

**Command Validation:**
- Performed in command thread before state update
- Returns `CommandError` for invalid parameters
- Sends `BackendEvent::CommandError` to frontend

**Common Errors:**
- `RowOutOfBounds` - Row index > 1
- `InvalidVolume` - Velocity outside 0.0-1.0
- `InvalidOctave` - Octave > 8
- `LockPoisoned` - Mutex/RwLock poisoned
- `NoAudioLoaded` - Query commands without loaded audio
- `InvalidTimeRange` - Time range outside buffer bounds

---

## Future Work

**Incomplete/TODO Areas:**

1. **Loop Commands** - No audio thread integration yet (need LoopAudioMessage variants)
2. **Mix Commands** - Could use `SetMasterVolume` AudioMessage for explicit control
3. **Performance Commands** - Effect state management incomplete (consider PerformanceAudioMessage)
4. **Effect Commands** - Audio processing not fully wired (consider EffectAudioMessage)
5. **Graph Commands** - `SaveGraph`/`LoadGraph` handled by frontend, `Swap` message unused
6. **Rustic Control** - `change_render_mode` incomplete in toolkit
7. **Graph System Cloning** - Render thread needs `System` clone (line 186 TODO)
8. **Master Volume Command** - Add command that generates `SetMasterVolume` message
9. **Sample Rate Command** - Add command for runtime sample rate changes (requires audio restart)

**Optimization Opportunities:**
- Lock-free state updates for hot-path parameters
- Batch audio message processing
- Ring buffer size tuning
- Graph topology pre-computation
- Direct AudioMessage API for low-latency control
