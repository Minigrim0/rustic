# Audio Graph System - Major Update Plan

## Context

Rustic has a mature DSP core (oscillators, filters, envelopes, instruments) and a 3-thread audio architecture (command thread → render thread → cpal callback), but the existing graph system (`System<INPUTS, OUTPUTS>` in `rustic/src/core/graph/system.rs`) is unused in the actual audio pipeline. The render thread simply sums all instrument outputs directly. The frontend graph editor tab is a hardcoded placeholder. The `Commands` enum has grown to ~50 flat variants with most having no audio translation.

**This update will** enable users to build audio graphs visually, placing generators/filters/sinks and running them through the real audio pipeline.

**Key architectural decisions:**
- Live playing stays instrument-based; graph editor uses the refactored System (two rendering modes coexist via `RenderMode` enum)
- Refactor existing `System` to use dynamic `Vec`s instead of const generics (no separate DynamicSystem)
- Runtime parameter changes use individual `AudioMessage` updates (real-time safe, no graph rebuild)
- Basic compressor filter implemented as part of this update

---

## Phase 1: Command Refactoring

### Goal
Split the flat `Commands` enum into grouped sub-enums to reduce complexity and make room for new `GraphCommand` variants.

### Step 1.1: Define the new command sub-enums

**File: `rustic/src/app/commands.rs`**

Replace the existing `Commands` enum with:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    System(SystemCommand),
    Live(LiveCommand),
    Loop(LoopCommand),
    Performance(PerformanceCommand),
    Mix(MixCommand),
    Effect(EffectCommand),
    Graph(GraphCommand),
    Settings(SettingsCommand),
}
```

Define each sub-enum in the same file:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemCommand {
    Quit,
    Reset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiveCommand {
    NoteStart { note: u8, row: u8, velocity: f32 },
    NoteStop { note: u8, row: u8 },
    OctaveUp(u8),
    OctaveDown(u8),
    SetOctave { octave: u8, row: u8 },
    LinkOctaves,
    UnlinkOctaves,
    SelectInstrument { index: usize, row: u8 },
    NextInstrument(u8),
    PreviousInstrument(u8),
    LinkInstruments,
    UnlinkInstruments,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoopCommand {
    StartRecording,
    StopRecording,
    PlayLoop,
    StopLoop,
    ClearLoop,
    LoopRepeat(bool),
    LoopRepeatCount(u32),
    SaveLoopToSlot(u8, u8),
    LoadLoopFromSlot(u8, u8),
    ClearLoopSlot(u8),
    ToggleLoopSlots(u8, u8),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceCommand {
    PitchBendUp(f32, u8),
    PitchBendDown(f32, u8),
    PitchBendReset(u8),
    Vibrato(f32, u8),
    Tremolo(f32, u8),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MixCommand {
    VolumeUp(u8),
    VolumeDown(u8),
    SetVolume(f32, u8),
    Mute(u8),
    MuteAll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectCommand {
    Reverb(f32, u8),
    Delay(f32, f32, u8),
    Chorus(f32, u8),
    Filter(f32, f32, u8),
    ToggleDistortion(u8),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphCommand {
    AddNode { node_type: String, kind: NodeKind, position: (f32, f32) },
    RemoveNode { id: u64 },
    Connect { from: u64, from_port: usize, to: u64, to_port: usize },
    Disconnect { from: u64, to: u64 },
    SetParameter { node_id: u64, param_name: String, value: f32 },
    SetNodePosition { id: u64, position: (f32, f32) },
    Play,
    Pause,
    Stop,
    SaveGraph(String),
    LoadGraph(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SettingsCommand {
    SwitchKeyboardLayout(String),
    ToggleHelp,
    Undo,
    Redo,
    TakeSnapshot,
    RestoreSnapshot(usize),
    LinkAll,
    UnlinkAll,
    SwapRows,
    CopyRowSettings(u8, u8),
    ToggleMetronome,
    SetTempo(u32),
    TempoUp,
    TempoDown,
    StartSessionRecording,
    StopSessionRecording,
    PlaySession,
    StopSession,
    SaveSession(String),
    LoadSession(String),
    ListOutputDevices,
    SelectOutputDevice(String),
}
```

### Step 1.2: Add temporary type alias

At the bottom of `rustic/src/app/commands.rs`, add:
```rust
/// Temporary alias for migration. Remove once all callsites are updated.
pub type Commands = Command;
```

### Step 1.3: Move `validate()` to dispatch to sub-enums

Refactor `Command::validate()` to delegate:
```rust
impl Command {
    pub fn validate(&self, app: &App) -> Result<(), CommandError> {
        match self {
            Command::Live(cmd) => cmd.validate(app),
            Command::System(_) => Ok(()),
            // ... all others Ok(()) for now
        }
    }
}

impl LiveCommand {
    pub fn validate(&self, _app: &App) -> Result<(), CommandError> {
        match self {
            LiveCommand::NoteStart { row, velocity, .. } => {
                if *row >= 2 { return Err(CommandError::RowOutOfBounds(*row)); }
                if *velocity < 0.0 || *velocity > 1.0 { return Err(CommandError::InvalidVolume(*velocity)); }
                Ok(())
            }
            LiveCommand::NoteStop { row, .. } => {
                if *row >= 2 { return Err(CommandError::RowOutOfBounds(*row)); }
                Ok(())
            }
            LiveCommand::SetOctave { octave, row } => {
                if *row >= 2 { return Err(CommandError::RowOutOfBounds(*row)); }
                if *octave > 8 { return Err(CommandError::InvalidOctave(*octave)); }
                Ok(())
            }
            LiveCommand::OctaveUp(row) | LiveCommand::OctaveDown(row) => {
                if *row >= 2 { return Err(CommandError::RowOutOfBounds(*row)); }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
```

### Step 1.4: Move `translate_to_audio_message()` to dispatch

Same dispatch pattern. Only `LiveCommand::NoteStart/NoteStop/OctaveUp/OctaveDown/SetOctave` and `SystemCommand::Quit` produce `AudioMessage`.

### Step 1.5: Update `App::on_event()` in `rustic/src/app/app.rs`

Change all match arms from `Commands::NoteStart(note, row, force)` to `Command::Live(LiveCommand::NoteStart { note, row, velocity })` etc.

### Step 1.6: Update `rustic/src/app/mod.rs` prelude

Change `pub use super::commands::Commands;` to `pub use super::commands::{Command, Commands};` (keeping alias).

### Step 1.7: Update `rustic/src/audio/command_thread.rs`

Change `Commands::Quit` match to `Command::System(SystemCommand::Quit)`.

### Step 1.8: Update `rustic/src/lib.rs` prelude

Update re-exports to include `Command`.

### Step 1.9: Update frontend files

**`frontend/src/tabs/mod.rs`:** Change `Tab` trait signature from `Sender<Commands>` to `Sender<Command>`.

**`frontend/src/main.rs`:**
- Change all `Commands` type annotations to `Command`
- Update channel types: `Sender<Command>`, `Receiver<Command>`

**`frontend/src/mapping.rs`:**
- Change all `Commands::NoteStart(...)` to `Command::Live(LiveCommand::NoteStart { ... })`
- Change `Commands::OctaveUp(...)` to `Command::Live(LiveCommand::OctaveUp(...))`
- etc.

**`frontend/src/tabs/live_playing.rs`**, **`frontend/src/tabs/settings.rs`:** Update command references.

### Step 1.10: Update all tests

**`rustic/tests/integration/commands.rs`:**
- Change all `Commands::NoteStart(0, 0, 0.5)` to `Command::Live(LiveCommand::NoteStart { note: 0, row: 0, velocity: 0.5 })`
- Same for NoteStop, SetOctave, OctaveUp, OctaveDown, Quit, Reset, etc.
- Update `commands_to_test` vectors
- Update `commands_without_audio_msg` vectors

### Step 1.11: Remove temporary alias

Once everything compiles and tests pass, remove `pub type Commands = Command;` from commands.rs and update any remaining references.

### Verification
```bash
cargo build --manifest-path rustic/Cargo.toml
cargo test --manifest-path rustic/Cargo.toml
cargo build --manifest-path frontend/Cargo.toml
```

---

## Phase 2: Complete Filter Metadata + Compressor

### Goal
Add `#[filter_parameter]` annotations to all filters missing them, implement the compressor, and update the metadata registry.

### Step 2.1: Fix `ResonantBandpassFilter` metadata

**File: `rustic/src/core/filters/resonant_bandpass.rs`**

Currently stores pre-computed coefficients `b: [f64; 3]` and `a: [f64; 3]` but not user-facing parameters. Need to:

1. Add stored fields with metadata annotations:
```rust
pub struct ResonantBandpassFilter {
    #[cfg_attr(feature = "meta", filter_source)]
    source: f32,
    index: usize,
    #[cfg_attr(feature = "meta", filter_parameter(range, 20.0, 20000.0, 1000.0))]
    center_frequency: f32,
    #[cfg_attr(feature = "meta", filter_parameter(range, 0.1, 20.0, 1.0))]
    quality: f32,
    #[cfg_attr(feature = "meta", filter_parameter(range, 8000.0, 96000.0, 44100.0))]
    sample_frequency: f32,
    b: [f64; 3],
    a: [f64; 3],
    zs: [f64; 2],
}
```

2. Add `recalculate_coefficients(&mut self)` method that recomputes `b` and `a` from the stored `center_frequency`, `quality`, `sample_frequency` (extract logic from current `new()`).

3. Update `new()` to store parameters and call `recalculate_coefficients()`.

4. Update `Default` impl to set reasonable defaults for the user-facing fields.

### Step 2.2: Add metadata to `MovingAverage`

**File: `rustic/src/core/filters/moving_average.rs`**

Add `#[filter_parameter]` to `size`. Since `size` is `usize` but the parameter system uses `f32`, we need to handle casting. The simplest approach: keep `size: usize` for internal use but add a separate `window_size: f32` field with the annotation, and cast in a setter:

```rust
pub struct MovingAverage {
    index: usize,
    #[cfg_attr(feature = "meta", filter_parameter(range, 1.0, 1024.0, 8.0))]
    window_size: f32,
    size: usize,
    buffer: Vec<f32>,
    #[cfg_attr(feature = "meta", filter_source)]
    source: f32,
}
```

Update `new()` and the `Default` impl to initialize `window_size` alongside `size`.

### Step 2.3: Implement Compressor

**File: `rustic/src/core/filters/compressor.rs`** (currently empty)

```rust
use std::fmt;
#[cfg(feature = "meta")]
use rustic_derive::FilterMetaData;
use crate::core::graph::{AudioGraphElement, Entry, Filter};

/// A dynamics compressor that reduces the dynamic range of audio signals.
/// When the input exceeds the threshold, the output is compressed by the given ratio.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "meta", derive(FilterMetaData))]
pub struct Compressor {
    #[cfg_attr(feature = "meta", filter_source)]
    source: f32,
    index: usize,
    /// Threshold in linear amplitude (0.0-1.0)
    #[cfg_attr(feature = "meta", filter_parameter(range, 0.0, 1.0, 0.5))]
    threshold: f32,
    /// Compression ratio (1.0 = no compression, higher = more compression)
    #[cfg_attr(feature = "meta", filter_parameter(range, 1.0, 20.0, 4.0))]
    ratio: f32,
    /// Attack time in seconds
    #[cfg_attr(feature = "meta", filter_parameter(range, 0.0001, 0.1, 0.01))]
    attack: f32,
    /// Release time in seconds
    #[cfg_attr(feature = "meta", filter_parameter(range, 0.01, 1.0, 0.1))]
    release: f32,
    /// Current envelope follower value
    envelope: f32,
    /// Sample rate for time-based calculations
    sample_rate: f32,
}

impl Default for Compressor {
    fn default() -> Self {
        Self {
            source: 0.0,
            index: 0,
            threshold: 0.5,
            ratio: 4.0,
            attack: 0.01,
            release: 0.1,
            envelope: 0.0,
            sample_rate: 44100.0,
        }
    }
}
```

Implement `new(threshold, ratio, attack, release, sample_rate)`, `Entry`, `Filter`, `AudioGraphElement`, `Display` traits.

The `transform()` logic:
1. Calculate attack/release coefficients: `attack_coeff = (-1.0 / (attack * sample_rate)).exp()`
2. Track envelope follower (peak detection)
3. Calculate gain reduction: if `envelope > threshold`, `gain = threshold + (envelope - threshold) / ratio`; normalize
4. Apply gain to source

### Step 2.4: Update filters `mod.rs`

**File: `rustic/src/core/filters/mod.rs`**

Add `mod compressor;` and add `pub use super::compressor::*;` to the prelude.

### Step 2.5: Update metadata registry

**File: `rustic/src/meta/mod.rs`**

Update `get_filters()` to return ALL filters with metadata:
```rust
pub fn get_filters() -> Vec<MetaFilter> {
    vec![
        crate::core::filters::prelude::GainFilter_META(),
        crate::core::filters::prelude::Clipper_META(),
        crate::core::filters::prelude::CombinatorFilter_META(),
        crate::core::filters::prelude::Tremolo_META(),
        crate::core::filters::prelude::DelayFilter_META(),
        crate::core::filters::prelude::LowPassFilter_META(),
        crate::core::filters::prelude::HighPassFilter_META(),
        crate::core::filters::prelude::BandPass_META(),
        crate::core::filters::prelude::ResonantBandpassFilter_META(),
        crate::core::filters::prelude::MovingAverage_META(),
        crate::core::filters::prelude::DuplicateFilter_META(),
        crate::core::filters::prelude::Compressor_META(),
    ]
}
```

### Verification
```bash
cargo build --manifest-path rustic/Cargo.toml --features meta
cargo test --manifest-path rustic/Cargo.toml --features meta
```

---

## Phase 3: Generator Metadata

### Goal
Create metadata for generators so they appear in the graph editor palette.

### Step 3.1: Add `NodeKind` enum

**File: `rustic/src/meta/traits.rs`**

Add at the top:
```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    Generator,
    Filter,
    Sink,
}
```

### Step 3.2: Add `MetaGenerator` struct

**File: `rustic/src/meta/structs.rs`**

Add:
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetaGenerator {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: Vec<Parameter<&'static str>>,
    pub output_count: usize,
}
```

### Step 3.3: Add `get_generators()` function

**File: `rustic/src/meta/mod.rs`**

Add a function that returns metadata for each waveform type. These are manually defined (no derive macro needed since there are only a handful):

```rust
use structs::MetaGenerator;
use rustic_meta::Parameter;

pub fn get_generators() -> Vec<MetaGenerator> {
    vec![
        MetaGenerator {
            name: "Sine Wave",
            description: "Generates a pure sine wave signal",
            parameters: vec![
                Parameter::Range {
                    title: "Frequency",
                    field_name: "frequency",
                    min: 20.0, max: 20000.0, default: 440.0, value: 440.0,
                },
                Parameter::Range {
                    title: "Amplitude",
                    field_name: "amplitude",
                    min: 0.0, max: 1.0, default: 0.5, value: 0.5,
                },
            ],
            output_count: 1,
        },
        // Square Wave, Sawtooth Wave, Triangle Wave, White Noise
        // Same structure, different name/description
    ]
}
```

### Step 3.4: Add `MetaSink` struct

**File: `rustic/src/meta/structs.rs`**

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetaSink {
    pub name: &'static str,
    pub description: &'static str,
    pub input_count: usize,
}
```

### Step 3.5: Add `get_sinks()` function

**File: `rustic/src/meta/mod.rs`**

```rust
pub fn get_sinks() -> Vec<MetaSink> {
    vec![
        MetaSink {
            name: "Audio Output",
            description: "Outputs audio to the system audio device",
            input_count: 1,
        },
    ]
}
```

### Verification
```bash
cargo build --manifest-path rustic/Cargo.toml --features meta
cargo test --manifest-path rustic/Cargo.toml --features meta
```

---

## Phase 4: Refactor System to Dynamic I/O

### Goal
Replace `System<const INPUTS, const OUTPUTS>` with a dynamic `System` using `Vec`s for sources and sinks.

### Step 4.1: Refactor `System` struct

**File: `rustic/src/core/graph/system.rs`**

Replace:
```rust
pub struct System<const INPUTS: usize, const OUTPUTS: usize> {
    graph: Graph<Box<dyn Filter>, (usize, usize)>,
    layers: Vec<Vec<usize>>,
    sources: [(Box<dyn Source>, (NodeIndex<u32>, usize)); INPUTS],
    sinks: [((NodeIndex<u32>, usize), Box<dyn Sink>); OUTPUTS],
}
```

With:
```rust
pub struct System {
    graph: Graph<Box<dyn Filter>, (usize, usize)>,
    layers: Vec<Vec<usize>>,
    sources: Vec<(Box<dyn Source>, (NodeIndex<u32>, usize))>,
    sinks: Vec<((NodeIndex<u32>, usize), Box<dyn Sink>)>,
}
```

### Step 4.2: Update `System::new()`

Replace the `core::array::from_fn` initialization with empty Vecs:
```rust
impl System {
    pub fn new() -> Self {
        System {
            graph: Graph::new(),
            layers: Vec::new(),
            sources: Vec::new(),
            sinks: Vec::new(),
        }
    }
}
```

Remove the `Default` impl that delegates to `new()` — or keep it as a simple delegate.

### Step 4.3: Add dynamic add/remove methods

```rust
impl System {
    /// Adds a source and returns its index
    pub fn add_source(&mut self, source: Box<dyn Source>) -> usize {
        let idx = self.sources.len();
        self.sources.push((source, (NodeIndex::new(0), 0)));
        idx
    }

    /// Removes a source by index
    pub fn remove_source(&mut self, index: usize) -> Option<Box<dyn Source>> {
        if index < self.sources.len() {
            Some(self.sources.remove(index).0)
        } else {
            None
        }
    }

    /// Adds a sink and returns its index
    pub fn add_sink(&mut self, sink: Box<dyn Sink>) -> usize {
        let idx = self.sinks.len();
        self.sinks.push(((NodeIndex::new(0), 0), sink));
        idx
    }

    /// Removes a sink by index
    pub fn remove_sink(&mut self, index: usize) -> Option<Box<dyn Sink>> {
        if index < self.sinks.len() {
            Some(self.sinks.remove(index).1)
        } else {
            None
        }
    }

    /// Removes a filter from the graph
    pub fn remove_filter(&mut self, index: NodeIndex<u32>) -> Option<Box<dyn Filter>> {
        self.graph.remove_node(index)
    }

    /// Disconnects two filters
    pub fn disconnect(&mut self, from: NodeIndex<u32>, to: NodeIndex<u32>) {
        if let Some(edge) = self.graph.find_edge(from, to) {
            self.graph.remove_edge(edge);
        }
    }
}
```

### Step 4.4: Update `set_source` and `set_sink`

Keep existing `set_source()` and `set_sink()` methods but add bounds checking:
```rust
pub fn set_source(&mut self, index: usize, source: Box<dyn Source>) {
    if index < self.sources.len() {
        self.sources[index] = (source, (NodeIndex::new(0), 0));
    } else {
        // Pad with empty sources if needed, or panic
        panic!("Source index {} out of bounds (len={})", index, self.sources.len());
    }
}
```

### Step 4.5: Update `merge()` method

Remove const generics from signature:
```rust
pub fn merge(mut self, other: System, mapping: Vec<(usize, usize)>) -> System {
    // ... same logic but use Vec operations instead of array operations
    // Replace core::array::from_fn with Vec construction
    let new_sinks: Vec<_> = other.sinks.iter().enumerate().map(|(index, sink)| {
        (
            (new_edge_map[&sink.0.0], sink.0.1),
            dyn_clone::clone_box(&*sink.1),
        )
    }).collect();

    System {
        graph: self.graph,
        layers: self.layers,
        sources: self.sources,
        sinks: new_sinks,
    }
}
```

### Step 4.6: Update `compute()` and `run()`

These methods don't use const generics internally - they iterate over `self.sources` and `self.sinks` which were already iterable. The only change needed is removing the const generic constraint from `impl<const INPUTS, const OUTPUTS>` blocks.

### Step 4.7: Update existing tests

**File: `rustic/tests/unit/core/graph/mod.rs`**

Change all `System::<1, 1>::new()` to `System::new()`, then use `add_source()` and `add_sink()` instead of `set_source(0, ...)` and `set_sink(0, ...)`:

```rust
// Before:
let mut system = System::<1, 1>::new();
system.set_source(0, source);
system.set_sink(0, Box::new(sink));

// After:
let mut system = System::new();
let src_idx = system.add_source(source);
let sink_idx = system.add_sink(Box::new(sink));
system.connect_source(src_idx, filt_1, 0);
system.connect_sink(filt_2, sink_idx, 0);
```

Update all 3 test functions: `test_system`, `stress_test`, `stress_test_2`.

### Step 4.8: Update `mod.rs` exports

**File: `rustic/src/core/graph/mod.rs`**

No changes needed to exports since `System` is already exported (just loses the generic parameters).

### Step 4.9: Add `Send + Sync` bounds where needed

For the graph to be sent between threads (render thread), ensure traits have appropriate bounds. Check if `Source`, `Filter`, `Sink` trait objects need `Send` bounds. Add `+ Send` to `Box<dyn Filter>` in System if needed.

### Verification
```bash
cargo build --manifest-path rustic/Cargo.toml
cargo test --manifest-path rustic/Cargo.toml
# Specifically run graph tests:
cargo test --manifest-path rustic/Cargo.toml -- core::graph
```

---

## Phase 5: Audio Output Sink

### Goal
Create a sink that writes to the cpal ring buffer, enabling graph-based audio output.

### Step 5.1: Create `AudioOutputSink`

**Create file: `rustic/src/core/graph/audio_sink.rs`**

```rust
use crossbeam::queue::ArrayQueue;
use std::sync::Arc;
use crate::core::graph::{AudioGraphElement, Entry, Sink};

/// A sink that writes audio samples to the cpal ring buffer for playback
#[derive(Clone, Debug)]
pub struct AudioOutputSink {
    values: Vec<f32>,
    index: usize,
}

impl Default for AudioOutputSink {
    fn default() -> Self {
        Self { values: Vec::new(), index: 0 }
    }
}

impl AudioOutputSink {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Entry for AudioOutputSink {
    fn push(&mut self, value: f32, _port: usize) {
        self.values.push(value);
    }
}

impl Sink for AudioOutputSink {
    fn consume(&mut self, amount: usize) -> Vec<f32> {
        let amount = std::cmp::min(amount, self.values.len());
        self.values.drain(0..amount).collect()
    }

    fn get_values(&self) -> Vec<f32> {
        self.values.clone()
    }

    fn into_entry(self) -> Box<dyn Entry> {
        Box::new(self)
    }
}

impl AudioGraphElement for AudioOutputSink {
    fn get_name(&self) -> &str { "Audio Output" }
    fn get_index(&self) -> usize { self.index }
    fn set_index(&mut self, index: usize) { self.index = index; }
}
```

Note: The actual writing to the ring buffer happens in the render thread (Phase 7), not in the sink itself. The sink collects samples via `consume()`, and the render thread writes them to the ring buffer. This keeps the sink decoupled from the audio backend.

### Step 5.2: Register the module

**File: `rustic/src/core/graph/mod.rs`**

Add:
```rust
mod audio_sink;
pub use audio_sink::AudioOutputSink;
```

### Step 5.3: Add device enumeration to backend events

**File: `rustic/src/audio/events.rs`**

Add variants:
```rust
pub enum BackendEvent {
    // ... existing
    OutputDeviceList { devices: Vec<String> },
    OutputDeviceChanged { device: String },
    GraphStateUpdated { /* will be defined in Phase 6 */ },
    GraphError { description: String },
}
```

### Verification
```bash
cargo build --manifest-path rustic/Cargo.toml
cargo test --manifest-path rustic/Cargo.toml
```

---

## Phase 6: Graph State Management

### Goal
Add serializable graph state to the `App` struct, handle `GraphCommand` in `on_event()`, and define the state-to-System conversion.

### Step 6.1: Create graph state types

**Create file: `rustic/src/app/graph_state.rs`**

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphState {
    pub nodes: Vec<NodeState>,
    pub connections: Vec<ConnectionState>,
    pub is_playing: bool,
    next_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub id: u64,
    pub node_type: String,      // matches metadata name (e.g., "Sine Wave", "Low Pass Filter")
    pub kind: NodeKind,         // Generator, Filter, Sink
    pub parameters: Vec<ParameterState>,
    pub position: (f32, f32),   // UI position for frontend
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionState {
    pub from_node: u64,
    pub from_port: usize,
    pub to_node: u64,
    pub to_port: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterState {
    pub name: String,
    pub value: f32,
}

// Re-use NodeKind from meta traits
use crate::meta::traits::NodeKind;
```

Add methods on `GraphState`:
```rust
impl GraphState {
    pub fn add_node(&mut self, node_type: String, kind: NodeKind, position: (f32, f32), default_params: Vec<ParameterState>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(NodeState {
            id, node_type, kind, parameters: default_params, position,
        });
        id
    }

    pub fn remove_node(&mut self, id: u64) {
        self.nodes.retain(|n| n.id != id);
        self.connections.retain(|c| c.from_node != id && c.to_node != id);
    }

    pub fn connect(&mut self, from: u64, from_port: usize, to: u64, to_port: usize) {
        self.connections.push(ConnectionState { from_node: from, from_port, to_node: to, to_port });
    }

    pub fn disconnect(&mut self, from: u64, to: u64) {
        self.connections.retain(|c| !(c.from_node == from && c.to_node == to));
    }

    pub fn set_parameter(&mut self, node_id: u64, param_name: &str, value: f32) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
            if let Some(param) = node.parameters.iter_mut().find(|p| p.name == param_name) {
                param.value = value;
            }
        }
    }

    pub fn find_node(&self, id: u64) -> Option<&NodeState> {
        self.nodes.iter().find(|n| n.id == id)
    }
}
```

### Step 6.2: Register module

**File: `rustic/src/app/mod.rs`**

Add:
```rust
#[cfg(feature = "meta")]
pub mod graph_state;
```

Update prelude:
```rust
pub mod prelude {
    pub use super::app::{App, AppMode, RunMode};
    pub use super::cli::Cli;
    pub use super::commands::{Command, Commands}; // Command + alias
    pub use super::filesystem::FSConfig;
    pub use super::system::SystemConfig;
    #[cfg(feature = "meta")]
    pub use super::graph_state::*;
}
```

### Step 6.3: Add `graph_state` to `App`

**File: `rustic/src/app/app.rs`**

Add field to `App`:
```rust
pub struct App {
    pub config: AppConfig,
    pub run_mode: RunMode,
    pub mode: AppMode,
    pub rows: [Row; 2],
    pub instruments: Vec<Box<dyn Instrument + Send + Sync>>,
    pub buffer: Vec<f32>,
    #[cfg(feature = "meta")]
    pub graph_state: GraphState,
}
```

Initialize in `Default::default()` and `new()`:
```rust
#[cfg(feature = "meta")]
graph_state: GraphState::default(),
```

### Step 6.4: Handle `GraphCommand` in `on_event()`

**File: `rustic/src/app/app.rs`**

Add match arm in `on_event()`:
```rust
Command::Graph(graph_cmd) => {
    #[cfg(feature = "meta")]
    match graph_cmd {
        GraphCommand::AddNode { node_type, kind, position } => {
            let default_params = self.get_default_params_for_node(&node_type);
            self.graph_state.add_node(node_type, kind, position, default_params);
        }
        GraphCommand::RemoveNode { id } => {
            self.graph_state.remove_node(id);
        }
        GraphCommand::Connect { from, from_port, to, to_port } => {
            self.graph_state.connect(from, from_port, to, to_port);
        }
        GraphCommand::Disconnect { from, to } => {
            self.graph_state.disconnect(from, to);
        }
        GraphCommand::SetParameter { node_id, param_name, value } => {
            self.graph_state.set_parameter(node_id, &param_name, value);
        }
        GraphCommand::SetNodePosition { id, position } => {
            if let Some(node) = self.graph_state.nodes.iter_mut().find(|n| n.id == id) {
                node.position = position;
            }
        }
        GraphCommand::Play => { self.graph_state.is_playing = true; }
        GraphCommand::Pause | GraphCommand::Stop => { self.graph_state.is_playing = false; }
        _ => {}
    }
}
```

### Step 6.5: Add `GraphState → System` builder

**Create file: `rustic/src/app/graph_builder.rs`** (or add to `graph_state.rs`)

```rust
use crate::core::graph::{System, AudioOutputSink};
use crate::core::graph::simple_source;
use crate::core::generator::prelude::*;
use crate::core::generator::prelude::builder::*;
use crate::core::filters::prelude::*;

impl GraphState {
    /// Builds a System from the current state. Returns None if the graph is invalid.
    pub fn build_system(&self) -> Result<System, String> {
        let mut system = System::new();
        // Map node IDs to graph indices
        let mut node_map: HashMap<u64, NodeIndex<u32>> = HashMap::new();
        let mut source_map: HashMap<u64, usize> = HashMap::new();
        let mut sink_map: HashMap<u64, usize> = HashMap::new();

        // 1. Create all nodes
        for node in &self.nodes {
            match node.kind {
                NodeKind::Generator => {
                    let source = self.create_source(node)?;
                    let idx = system.add_source(source);
                    source_map.insert(node.id, idx);
                }
                NodeKind::Filter => {
                    let filter = self.create_filter(node)?;
                    let idx = system.add_filter(filter);
                    node_map.insert(node.id, idx);
                }
                NodeKind::Sink => {
                    let sink = Box::new(AudioOutputSink::new());
                    let idx = system.add_sink(sink);
                    sink_map.insert(node.id, idx);
                }
            }
        }

        // 2. Create connections
        for conn in &self.connections {
            let from_is_source = source_map.contains_key(&conn.from_node);
            let to_is_sink = sink_map.contains_key(&conn.to_node);

            if from_is_source && !to_is_sink {
                // Source → Filter
                let src_idx = source_map[&conn.from_node];
                let filter_idx = node_map[&conn.to_node];
                system.connect_source(src_idx, filter_idx, conn.to_port);
            } else if !from_is_source && to_is_sink {
                // Filter → Sink
                let filter_idx = node_map[&conn.from_node];
                let sink_idx = sink_map[&conn.to_node];
                system.connect_sink(filter_idx, sink_idx, conn.from_port);
            } else if !from_is_source && !to_is_sink {
                // Filter → Filter
                let from_idx = node_map[&conn.from_node];
                let to_idx = node_map[&conn.to_node];
                system.connect(from_idx, to_idx, conn.from_port, conn.to_port);
            }
            // Source → Sink directly is not supported (need at least one filter)
        }

        // 3. Compute topology
        system.compute().map_err(|_| "Graph contains cycles without postponable filters".to_string())?;

        Ok(system)
    }

    fn create_source(&self, node: &NodeState) -> Result<Box<dyn Source>, String> {
        let freq = node.parameters.iter()
            .find(|p| p.name == "frequency")
            .map(|p| p.value)
            .unwrap_or(440.0);

        let waveform = match node.node_type.as_str() {
            "Sine Wave" => Waveform::Sine,
            "Square Wave" => Waveform::Square,
            "Sawtooth Wave" => Waveform::Sawtooth,
            "Triangle Wave" => Waveform::Triangle,
            "White Noise" => Waveform::WhiteNoise,
            other => return Err(format!("Unknown generator type: {}", other)),
        };

        let mut gen: MultiToneGenerator = ToneGeneratorBuilder::new()
            .waveform(waveform)
            .frequency(freq)
            .build()
            .into();
        gen.start();

        Ok(simple_source(gen))
    }

    fn create_filter(&self, node: &NodeState) -> Result<Box<dyn Filter>, String> {
        match node.node_type.as_str() {
            "Low Pass Filter" => {
                let cutoff = node.get_param("cutoff_frequency").unwrap_or(1000.0);
                Ok(Box::new(LowPassFilter::new(cutoff)))
            }
            "High Pass Filter" => {
                let cutoff = node.get_param("cutoff_frequency").unwrap_or(1000.0);
                Ok(Box::new(HighPassFilter::new(cutoff)))
            }
            "Gain" => {
                let factor = node.get_param("factor").unwrap_or(1.0);
                Ok(Box::new(GainFilter::new(factor)))
            }
            // ... all other filter types
            other => Err(format!("Unknown filter type: {}", other)),
        }
    }
}

impl NodeState {
    pub fn get_param(&self, name: &str) -> Option<f32> {
        self.parameters.iter().find(|p| p.name == name).map(|p| p.value)
    }
}
```

### Verification
```bash
cargo build --manifest-path rustic/Cargo.toml --features meta
cargo test --manifest-path rustic/Cargo.toml --features meta
```

---

## Phase 7: Graph-Render Thread Integration

### Goal
Enable the render thread to execute a `System` graph alongside or instead of the instrument-based rendering.

### Step 7.1: Add new `AudioMessage` variants

**File: `rustic/src/audio/messages.rs`**

```rust
use crate::core::graph::System;

#[derive(Debug, Clone)]
pub enum AudioMessage {
    // ... existing variants (NoteStart, NoteStop, SetOctave, SetMasterVolume, SetSampleRate, Shutdown)

    // Graph management
    /// Replace the entire graph atomically
    SwapGraph(Box<System>),
    /// Clear the graph (stop graph-based rendering)
    ClearGraph,
    /// Set the rendering mode
    SetRenderMode(RenderMode),
    /// Update a filter parameter in the active graph (real-time safe)
    GraphSetParameter { node_index: usize, param_name: String, value: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderMode {
    /// Legacy: sum instruments directly (current behavior)
    Instruments,
    /// Use the System graph for rendering
    Graph,
}
```

Note: `System` needs to implement `Debug` (already does) and `Send` (needs verification - graph nodes must be `Send`). If `Box<dyn Filter>` isn't `Send`, add `Send` bound to the `Filter` trait or to `System`'s type parameters. The `Clone` derive on `AudioMessage` will require `System` to be cloneable OR we use a non-Clone wrapper. Since `AudioMessage` is sent through crossbeam channels (which don't require Clone), we can remove `#[derive(Clone)]` from `AudioMessage` or use an `Arc`.

**Resolution**: Change `AudioMessage` to not derive `Clone` (it's only sent, never cloned in practice). Or wrap `System` in `Arc`:
```rust
SwapGraph(Arc<Mutex<System>>),
```

Better approach: Use `Option<Box<System>>` and take ownership:
```rust
// Don't derive Clone on AudioMessage. Use move semantics.
pub enum AudioMessage {
    // ...
    SwapGraph(Box<System>),
    // ...
}
```

Check if any code clones `AudioMessage`. The command thread in `command_thread.rs` does `msg.clone()` on line 59. Fix this by removing the clone or restructuring.

### Step 7.2: Update render thread

**File: `rustic/src/audio/render_thread.rs`**

```rust
use crate::core::graph::System;
use super::messages::RenderMode;

pub fn spawn_audio_render_thread(
    shared_state: Arc<SharedAudioState>,
    instruments: Vec<Box<dyn Instrument + Send + Sync>>,
    message_rx: crossbeam::channel::Receiver<AudioMessage>,
    audio_queue: Arc<ArrayQueue<f32>>,
    config: AudioConfig,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("audio-render".to_string())
        .spawn(move || {
            let mut instruments = instruments;
            let mut graph: Option<System> = None;
            let mut render_mode = RenderMode::Instruments;
            let mut chunk_buffer = vec![0.0f32; config.render_chunk_size];

            while !shared_state.shutdown.load(Ordering::Relaxed) {
                // Process all pending control messages
                while let Ok(msg) = message_rx.try_recv() {
                    process_audio_message(
                        &mut instruments, &mut graph, &mut render_mode, msg
                    );
                }

                // Check if ring buffer has space
                if audio_queue.len() + config.render_chunk_size > audio_queue.capacity() {
                    thread::sleep(Duration::from_micros(100));
                    continue;
                }

                // Generate audio chunk based on render mode
                match render_mode {
                    RenderMode::Instruments => {
                        for sample in chunk_buffer.iter_mut() {
                            instruments.iter_mut().for_each(|inst| inst.tick());
                            *sample = instruments
                                .iter_mut()
                                .map(|inst| inst.get_output())
                                .sum::<f32>();
                        }
                    }
                    RenderMode::Graph => {
                        if let Some(ref mut system) = graph {
                            for sample in chunk_buffer.iter_mut() {
                                system.run();
                                if let Ok(sink) = system.get_sink(0) {
                                    let values = sink.consume(1);
                                    *sample = values.first().copied().unwrap_or(0.0);
                                } else {
                                    *sample = 0.0;
                                }
                            }
                        } else {
                            // No graph loaded, output silence
                            chunk_buffer.fill(0.0);
                        }
                    }
                }

                // Write to ring buffer (unchanged)
                let mut written = 0;
                for &sample in chunk_buffer.iter() {
                    if audio_queue.push(sample).is_ok() {
                        written += 1;
                    } else {
                        break;
                    }
                }
                if written != chunk_buffer.len() {
                    log::warn!("Failed to write full chunk: {} / {}", written, chunk_buffer.len());
                }
            }
            log::info!("Audio render thread shutting down");
        })
        .expect("Failed to spawn audio render thread")
}

fn process_audio_message(
    instruments: &mut [Box<dyn Instrument + Send + Sync>],
    graph: &mut Option<System>,
    render_mode: &mut RenderMode,
    msg: AudioMessage,
) {
    match msg {
        // ... existing NoteStart, NoteStop, etc. (unchanged)

        AudioMessage::SwapGraph(new_graph) => {
            *graph = Some(*new_graph);
            log::info!("Graph swapped successfully");
        }
        AudioMessage::ClearGraph => {
            *graph = None;
            log::info!("Graph cleared");
        }
        AudioMessage::SetRenderMode(mode) => {
            *render_mode = mode;
            log::info!("Render mode changed to {:?}", mode);
        }
        AudioMessage::GraphSetParameter { node_index, param_name, value } => {
            // TODO: Apply parameter change to the active graph
            // This requires filter parameter setter methods
            if let Some(ref mut system) = graph {
                // system.set_filter_parameter(node_index, &param_name, value);
            }
        }
        // ... existing handlers
    }
}
```

### Step 7.3: Update command thread for graph commands

**File: `rustic/src/audio/command_thread.rs`**

In the main `match` block, after updating app state via `on_event()`, translate `GraphCommand` to `AudioMessage`:

```rust
Ok(cmd) => {
    // Validate
    if let Err(e) = cmd.validate(&app) { /* ... */ continue; }

    // Update app state
    app.on_event(cmd.clone());

    // Translate to audio message
    match &cmd {
        Command::Graph(GraphCommand::Play) => {
            // Build system from graph state
            #[cfg(feature = "meta")]
            match app.graph_state.build_system() {
                Ok(system) => {
                    let _ = message_tx.send(AudioMessage::SwapGraph(Box::new(system)));
                    let _ = message_tx.send(AudioMessage::SetRenderMode(RenderMode::Graph));
                }
                Err(e) => {
                    let _ = event_tx.send(BackendEvent::GraphError { description: e });
                }
            }
        }
        Command::Graph(GraphCommand::Stop) => {
            let _ = message_tx.send(AudioMessage::SetRenderMode(RenderMode::Instruments));
            let _ = message_tx.send(AudioMessage::ClearGraph);
        }
        Command::Graph(GraphCommand::SetParameter { node_id, param_name, value }) => {
            // Send real-time parameter update
            // Need to map node_id to graph node_index
            let _ = message_tx.send(AudioMessage::GraphSetParameter {
                node_index: *node_id as usize, // simplified mapping
                param_name: param_name.clone(),
                value: *value,
            });
        }
        _ => {
            // Existing translation logic
            if let Some(msg) = cmd.translate_to_audio_message(&mut app) {
                let _ = message_tx.send(msg);
            }
        }
    }
}
```

### Step 7.4: Ensure `Send` bounds

Add `Send` to trait objects where needed. In `rustic/src/core/graph/mod.rs`, the traits may need `Send` bounds:
```rust
pub trait Source: AudioGraphElement + Send { ... }
pub trait Filter: Entry + AudioGraphElement + fmt::Display + fmt::Debug + Send { ... }
pub trait Sink: Entry + AudioGraphElement + Send { ... }
pub trait Entry: AudioGraphElement + DynClone + Send { ... }
```

This may require updating all filter, source, and sink implementations if they hold non-Send types. Check each one.

### Verification
```bash
cargo build --manifest-path rustic/Cargo.toml --features meta
cargo test --manifest-path rustic/Cargo.toml --features meta
cargo build --manifest-path frontend/Cargo.toml
```

---

## Phase 8: Frontend Graph Editor Integration

### Goal
Connect the placeholder `GraphEditorTab` to the actual backend, with metadata-driven palette, node interactions, and play/pause controls.

### Step 8.1: Update `Tab` trait

**File: `frontend/src/tabs/mod.rs`**

Change trait to accept `BackendEvent` receiver:
```rust
use rustic::audio::BackendEvent;
use std::sync::mpsc::Receiver;

pub trait Tab {
    fn ui(&mut self, ui: &mut Ui, app_sender: &Sender<Command>);

    /// Process any pending backend events. Default impl does nothing.
    fn process_events(&mut self, _receiver: &Receiver<BackendEvent>) {}
}
```

### Step 8.2: Replace `GraphEditorTab` state

**File: `frontend/src/tabs/graph_editor.rs`**

Replace the hardcoded demo with metadata-driven state:

```rust
use rustic::prelude::Command;
use rustic::app::commands::{GraphCommand, NodeKind};
use rustic::meta::{get_filters, get_generators, get_sinks};

pub struct GraphEditorTab {
    // Palette (populated from metadata)
    generator_entries: Vec<PaletteEntry>,
    filter_entries: Vec<PaletteEntry>,
    sink_entries: Vec<PaletteEntry>,

    // Graph state (mirrors backend GraphState)
    nodes: Vec<GraphNode>,
    connections: Vec<(u64, u64)>,  // (from_id, to_id)

    // UI state
    selected_node: Option<u64>,
    is_playing: bool,
    canvas_offset: Vec2,
    zoom: f32,
    show_help: bool,

    // Connection dragging
    dragging_from: Option<(u64, bool)>,  // (node_id, is_output)
}

struct PaletteEntry {
    name: String,
    kind: NodeKind,
    description: String,
}

struct GraphNode {
    id: u64,
    name: String,
    kind: NodeKind,
    position: Vec2,
    params: Vec<EditableParam>,
    input_ports: usize,
    output_ports: usize,
}

struct EditableParam {
    name: String,
    value: f32,
    min: f32,
    max: f32,
}
```

### Step 8.3: Populate palette from metadata

In `GraphEditorTab::new()`:
```rust
pub fn new() -> Self {
    let generator_entries: Vec<PaletteEntry> = get_generators().into_iter().map(|g| {
        PaletteEntry { name: g.name.to_string(), kind: NodeKind::Generator, description: g.description.to_string() }
    }).collect();

    let filter_entries: Vec<PaletteEntry> = get_filters().into_iter().map(|f| {
        PaletteEntry { name: f.name.to_string(), kind: NodeKind::Filter, description: f.description.to_string() }
    }).collect();

    let sink_entries = vec![
        PaletteEntry { name: "Audio Output".to_string(), kind: NodeKind::Sink, description: "System audio output".to_string() },
    ];

    GraphEditorTab {
        generator_entries, filter_entries, sink_entries,
        nodes: Vec::new(),
        connections: Vec::new(),
        selected_node: None,
        is_playing: false,
        canvas_offset: Vec2::ZERO,
        zoom: 1.0,
        show_help: false,
        dragging_from: None,
    }
}
```

### Step 8.4: Wire palette clicks to commands

In `draw_palette()`, when a button is clicked:
```rust
if ui.button(&entry.name).clicked() {
    let pos = (200.0 + self.nodes.len() as f32 * 50.0, 200.0);
    app_sender.send(Command::Graph(GraphCommand::AddNode {
        node_type: entry.name.clone(),
        kind: entry.kind,
        position: pos,
    })).ok();
}
```

### Step 8.5: Add play/pause/stop controls

Add a toolbar above the canvas:
```rust
ui.horizontal(|ui| {
    if ui.button(if self.is_playing { "Stop" } else { "Play" }).clicked() {
        if self.is_playing {
            app_sender.send(Command::Graph(GraphCommand::Stop)).ok();
            self.is_playing = false;
        } else {
            app_sender.send(Command::Graph(GraphCommand::Play)).ok();
            self.is_playing = true;
        }
    }
});
```

### Step 8.6: Add parameter editing panel

When a node is selected, show its parameters as sliders:
```rust
if let Some(selected_id) = self.selected_node {
    if let Some(node) = self.nodes.iter_mut().find(|n| n.id == selected_id) {
        egui::Window::new("Node Properties").show(ui.ctx(), |ui| {
            ui.heading(&node.name);
            for param in &mut node.params {
                let response = ui.add(egui::Slider::new(&mut param.value, param.min..=param.max).text(&param.name));
                if response.changed() {
                    app_sender.send(Command::Graph(GraphCommand::SetParameter {
                        node_id: node.id,
                        param_name: param.name.clone(),
                        value: param.value,
                    })).ok();
                }
            }
        });
    }
}
```

### Step 8.7: Implement node dragging

In `draw_canvas()`, make nodes draggable by tracking mouse delta when a node is clicked and held.

### Step 8.8: Implement connection drawing

Allow users to drag from an output port to an input port to create connections. When released on a valid target, send `Command::Graph(GraphCommand::Connect { ... })`.

### Step 8.9: Synchronize state from backend

In `process_events()`:
```rust
fn process_events(&mut self, receiver: &Receiver<BackendEvent>) {
    while let Ok(event) = receiver.try_recv() {
        match event {
            BackendEvent::GraphStateUpdated { state } => {
                // Sync nodes and connections from backend state
                // This ensures frontend stays in sync after graph operations
            }
            BackendEvent::GraphError { description } => {
                log::error!("Graph error: {}", description);
                self.is_playing = false;
            }
            _ => {}
        }
    }
}
```

### Verification
```bash
cargo build --manifest-path frontend/Cargo.toml
# Manual testing:
cargo run --manifest-path frontend/Cargo.toml
# 1. Go to Graph Editor tab
# 2. Add Sine Wave from palette
# 3. Add Low Pass Filter
# 4. Add Audio Output
# 5. Connect nodes
# 6. Adjust cutoff frequency slider
# 7. Press Play → hear filtered sine
# 8. Press Stop → silence
# 9. Switch to Live Playing → keyboard still works
```

---

## Summary of All Files Modified/Created

### Modified files:
| File | Phase |
|------|-------|
| `rustic/src/app/commands.rs` | 1 |
| `rustic/src/app/app.rs` | 1, 6 |
| `rustic/src/app/mod.rs` | 1, 6 |
| `rustic/src/lib.rs` | 1 |
| `rustic/src/audio/command_thread.rs` | 1, 7 |
| `rustic/src/audio/messages.rs` | 7 |
| `rustic/src/audio/render_thread.rs` | 7 |
| `rustic/src/audio/events.rs` | 5 |
| `rustic/src/audio/mod.rs` | 7 |
| `rustic/src/core/graph/mod.rs` | 4, 5 |
| `rustic/src/core/graph/system.rs` | 4 |
| `rustic/src/core/filters/mod.rs` | 2 |
| `rustic/src/core/filters/resonant_bandpass.rs` | 2 |
| `rustic/src/core/filters/moving_average.rs` | 2 |
| `rustic/src/core/filters/compressor.rs` | 2 |
| `rustic/src/meta/mod.rs` | 2, 3 |
| `rustic/src/meta/structs.rs` | 3 |
| `rustic/src/meta/traits.rs` | 3 |
| `rustic/tests/integration/commands.rs` | 1 |
| `rustic/tests/unit/core/graph/mod.rs` | 4 |
| `frontend/src/main.rs` | 1, 8 |
| `frontend/src/mapping.rs` | 1 |
| `frontend/src/tabs/mod.rs` | 1, 8 |
| `frontend/src/tabs/graph_editor.rs` | 8 |
| `frontend/src/tabs/live_playing.rs` | 1 |
| `frontend/src/tabs/settings.rs` | 1 |

### New files:
| File | Phase |
|------|-------|
| `rustic/src/core/graph/audio_sink.rs` | 5 |
| `rustic/src/app/graph_state.rs` | 6 |
| `rustic/src/app/graph_builder.rs` | 6 (or inline in graph_state.rs) |


```bash
claude --resume b8777662-438e-42d5-b0da-d639f1da3622
```
