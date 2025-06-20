use sdl2::keyboard::{Keycode, Mod};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use rustic::prelude::Commands;

use super::keys::{KeyWithModifiers, ModifierState, SerializableKey};

/// Different trigger types for mappings
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriggerType {
    OnPress,
    OnRelease,
    OnBoth,
}

/// Runtime key pattern (optimized for performance)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyPattern {
    Single(KeyWithModifiers),
    Chord(Vec<KeyWithModifiers>),
    Sequence(Vec<KeyWithModifiers>),
}

/// Serializable key pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializablePattern {
    Single(SerializableKey),
    Chord(Vec<SerializableKey>),
    Sequence(Vec<SerializableKey>),
}

impl SerializablePattern {
    pub fn from_runtime(pattern: &KeyPattern) -> Self {
        match pattern {
            KeyPattern::Single(key) => {
                SerializablePattern::Single(SerializableKey::from_runtime(key))
            }
            KeyPattern::Chord(keys) => {
                SerializablePattern::Chord(keys.iter().map(SerializableKey::from_runtime).collect())
            }
            KeyPattern::Sequence(keys) => SerializablePattern::Sequence(
                keys.iter().map(SerializableKey::from_runtime).collect(),
            ),
        }
    }

    pub fn to_runtime(&self) -> Option<KeyPattern> {
        match self {
            SerializablePattern::Single(key) => key.to_runtime().map(KeyPattern::Single),
            SerializablePattern::Chord(keys) => {
                let runtime_keys: Option<Vec<_>> = keys.iter().map(|k| k.to_runtime()).collect();
                runtime_keys.map(KeyPattern::Chord)
            }
            SerializablePattern::Sequence(keys) => {
                let runtime_keys: Option<Vec<_>> = keys.iter().map(|k| k.to_runtime()).collect();
                runtime_keys.map(KeyPattern::Sequence)
            }
        }
    }
}

/// Runtime key mapping (optimized for performance)
#[derive(Debug, Clone)]
pub struct KeyMapping {
    pub pattern: KeyPattern,
    pub trigger: TriggerType,
    pub press_commands: Vec<Commands>,
    pub release_commands: Vec<Commands>,
    pub timeout_ms: Option<u64>,
}

/// Serializable key mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableMapping {
    pub pattern: SerializablePattern,
    pub trigger: TriggerType,
    pub press_commands: Vec<Commands>,
    pub release_commands: Vec<Commands>,
    pub timeout_ms: Option<u64>,
}

impl SerializableMapping {
    pub fn from_runtime(mapping: &KeyMapping) -> Self {
        Self {
            pattern: SerializablePattern::from_runtime(&mapping.pattern),
            trigger: mapping.trigger.clone(),
            press_commands: mapping.press_commands.clone(),
            release_commands: mapping.release_commands.clone(),
            timeout_ms: mapping.timeout_ms,
        }
    }

    pub fn to_runtime(&self) -> Option<KeyMapping> {
        self.pattern.to_runtime().map(|pattern| KeyMapping {
            pattern,
            trigger: self.trigger.clone(),
            press_commands: self.press_commands.clone(),
            release_commands: self.release_commands.clone(),
            timeout_ms: self.timeout_ms,
        })
    }
}

/// Tracks sequence input state
#[derive(Debug)]
struct SequenceState {
    pattern: Vec<KeyWithModifiers>,
    current_index: usize,
    last_input_time: Instant,
    timeout: Duration,
}

/// Configuration for the key mapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMapperConfig {
    pub default_sequence_timeout_ms: u64,
    pub enable_chord_detection: bool,
    pub enable_sequence_detection: bool,
}

impl Default for KeyMapperConfig {
    fn default() -> Self {
        Self {
            default_sequence_timeout_ms: 1000,
            enable_chord_detection: true,
            enable_sequence_detection: true,
        }
    }
}

/// High-performance key mapper optimized for <1ms latency
pub struct KeyMapper {
    // Fast O(1) lookups for single keys
    press_mappings: HashMap<KeyWithModifiers, Vec<Commands>>,
    release_mappings: HashMap<KeyWithModifiers, Vec<Commands>>,

    // Chord mappings (sorted by length for priority)
    chord_mappings: Vec<KeyMapping>,

    // Sequence mappings
    sequence_mappings: Vec<KeyMapping>,

    // Current state
    pressed_keys: HashSet<KeyWithModifiers>,
    active_sequences: Vec<SequenceState>,

    // Performance optimization sets
    chord_keys: HashSet<KeyWithModifiers>,
    sequence_keys: HashSet<KeyWithModifiers>,

    // Communication
    command_sender: Sender<Commands>,

    // Configuration
    config: KeyMapperConfig,
}

impl KeyMapper {
    pub fn new(command_sender: Sender<Commands>) -> Self {
        Self {
            press_mappings: HashMap::new(),
            release_mappings: HashMap::new(),
            chord_mappings: Vec::new(),
            sequence_mappings: Vec::new(),
            pressed_keys: HashSet::new(),
            active_sequences: Vec::new(),
            chord_keys: HashSet::new(),
            sequence_keys: HashSet::new(),
            command_sender,
            config: KeyMapperConfig::default(),
        }
    }

    /// Add a mapping to the system
    pub fn add_mapping(&mut self, mapping: KeyMapping) {
        match &mapping.pattern {
            KeyPattern::Single(key) => {
                match mapping.trigger {
                    TriggerType::OnPress | TriggerType::OnBoth => {
                        self.press_mappings
                            .entry(key.clone())
                            .or_insert_with(Vec::new)
                            .extend(mapping.press_commands.iter().cloned());
                    }
                    _ => {}
                }
                match mapping.trigger {
                    TriggerType::OnRelease | TriggerType::OnBoth => {
                        self.release_mappings
                            .entry(key.clone())
                            .or_insert_with(Vec::new)
                            .extend(mapping.release_commands.iter().cloned());
                    }
                    _ => {}
                }
            }
            KeyPattern::Chord(keys) => {
                if self.config.enable_chord_detection {
                    for key in keys {
                        self.chord_keys.insert(key.clone());
                    }
                    self.chord_mappings.push(mapping);
                    // Sort by length (longer chords have priority)
                    self.chord_mappings.sort_by_key(|m| {
                        if let KeyPattern::Chord(keys) = &m.pattern {
                            std::cmp::Reverse(keys.len())
                        } else {
                            std::cmp::Reverse(0)
                        }
                    });
                }
            }
            KeyPattern::Sequence(keys) => {
                if self.config.enable_sequence_detection {
                    for key in keys {
                        self.sequence_keys.insert(key.clone());
                    }
                    self.sequence_mappings.push(mapping);
                }
            }
        }
    }

    /// Handle key press with minimal latency
    pub fn handle_key_press(&mut self, keycode: Keycode, modifiers: Mod) {
        let key = KeyWithModifiers {
            keycode,
            modifiers: ModifierState::from_sdl_mod(modifiers),
        };

        self.pressed_keys.insert(key.clone());

        // 1. Direct press mapping (fastest path - O(1))
        if let Some(commands) = self.press_mappings.get(&key) {
            self.send_commands(commands);
        }

        // 2. Check chord completion (only if key participates in chords)
        if self.config.enable_chord_detection && self.chord_keys.contains(&key) {
            self.check_chord_completion();
        }

        // 3. Update sequences (only if key participates in sequences)
        if self.config.enable_sequence_detection && self.sequence_keys.contains(&key) {
            self.update_sequences(&key);
        }
    }

    /// Handle key release with minimal latency
    pub fn handle_key_release(&mut self, keycode: Keycode, modifiers: Mod) {
        let key = KeyWithModifiers {
            keycode,
            modifiers: ModifierState::from_sdl_mod(modifiers),
        };

        self.pressed_keys.remove(&key);

        // Direct release mapping (O(1))
        if let Some(commands) = self.release_mappings.get(&key) {
            self.send_commands(commands);
        }
    }

    /// Check if any chord is completed
    fn check_chord_completion(&mut self) {
        for mapping in &self.chord_mappings {
            if let KeyPattern::Chord(chord_keys) = &mapping.pattern {
                if chord_keys.iter().all(|key| self.pressed_keys.contains(key)) {
                    if let TriggerType::OnPress | TriggerType::OnBoth = mapping.trigger {
                        self.send_commands(&mapping.press_commands);
                    }
                    return; // Only trigger first matching chord
                }
            }
        }
    }

    /// Update sequence detection
    fn update_sequences(&mut self, key: &KeyWithModifiers) {
        let now = Instant::now();

        // Remove timed-out sequences
        self.active_sequences
            .retain(|seq| now.duration_since(seq.last_input_time) < seq.timeout);

        // Check existing sequences and collect completed patterns
        let mut completed_pattern = None;
        for seq in &mut self.active_sequences {
            if seq.current_index < seq.pattern.len() && &seq.pattern[seq.current_index] == key {
                seq.current_index += 1;
                seq.last_input_time = now;

                // Check completion
                if seq.current_index == seq.pattern.len() {
                    completed_pattern = Some(seq.pattern.clone());
                    break;
                }
            }
        }

        // Execute completed sequence (after borrowing ends)
        if let Some(pattern) = completed_pattern {
            self.execute_sequence(&pattern);
            return;
        }

        // Start new sequences
        for mapping in &self.sequence_mappings {
            if let KeyPattern::Sequence(pattern) = &mapping.pattern {
                if !pattern.is_empty() && &pattern[0] == key {
                    let timeout = Duration::from_millis(
                        mapping
                            .timeout_ms
                            .unwrap_or(self.config.default_sequence_timeout_ms),
                    );

                    self.active_sequences.push(SequenceState {
                        pattern: pattern.clone(),
                        current_index: 1,
                        last_input_time: now,
                        timeout,
                    });
                }
            }
        }
    }

    /// Execute completed sequence
    fn execute_sequence(&self, pattern: &[KeyWithModifiers]) {
        for mapping in &self.sequence_mappings {
            if let KeyPattern::Sequence(seq_pattern) = &mapping.pattern {
                if seq_pattern == pattern {
                    if let TriggerType::OnPress | TriggerType::OnBoth = mapping.trigger {
                        self.send_commands(&mapping.press_commands);
                    }
                    break;
                }
            }
        }
    }

    /// Send commands through channel
    fn send_commands(&self, commands: &[Commands]) {
        for command in commands {
            if self.command_sender.send(command.clone()).is_err() {
                eprintln!("Warning: Command channel closed");
            }
        }
    }

    /// Clear all mappings
    pub fn clear_mappings(&mut self) {
        self.press_mappings.clear();
        self.release_mappings.clear();
        self.chord_mappings.clear();
        self.sequence_mappings.clear();
        self.chord_keys.clear();
        self.sequence_keys.clear();
    }

    /// Configuration management
    pub fn config(&self) -> &KeyMapperConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: KeyMapperConfig) {
        self.config = config;
    }
}

/// File I/O operations
impl KeyMapper {
    /// Save mappings to file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut serializable_mappings = Vec::new();

        // Convert all mappings to serializable format
        for mapping in &self.chord_mappings {
            serializable_mappings.push(SerializableMapping::from_runtime(mapping));
        }

        for mapping in &self.sequence_mappings {
            serializable_mappings.push(SerializableMapping::from_runtime(mapping));
        }

        // Convert direct mappings
        for (key, commands) in &self.press_mappings {
            let mapping = KeyMapping {
                pattern: KeyPattern::Single(key.clone()),
                trigger: TriggerType::OnPress,
                press_commands: commands.clone(),
                release_commands: vec![],
                timeout_ms: None,
            };
            serializable_mappings.push(SerializableMapping::from_runtime(&mapping));
        }

        for (key, commands) in &self.release_mappings {
            let mapping = KeyMapping {
                pattern: KeyPattern::Single(key.clone()),
                trigger: TriggerType::OnRelease,
                press_commands: vec![],
                release_commands: commands.clone(),
                timeout_ms: None,
            };
            serializable_mappings.push(SerializableMapping::from_runtime(&mapping));
        }

        let config_data = ConfigFile {
            config: self.config.clone(),
            mappings: serializable_mappings,
        };

        let toml_content = toml::to_string_pretty(&config_data)?;
        std::fs::write(path, toml_content)?;
        Ok(())
    }

    /// Load mappings from file
    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let toml_content = std::fs::read_to_string(path)?;
        let config_data: ConfigFile = toml::from_str(&toml_content)?;

        self.clear_mappings();
        self.config = config_data.config;

        for serializable_mapping in config_data.mappings {
            if let Some(runtime_mapping) = serializable_mapping.to_runtime() {
                self.add_mapping(runtime_mapping);
            } else {
                eprintln!("Warning: Failed to load mapping with invalid keycode");
            }
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct ConfigFile {
    config: KeyMapperConfig,
    mappings: Vec<SerializableMapping>,
}

/// Helper functions for common operations
impl KeyMapper {
    /// Create modifier state with no modifiers
    pub fn no_modifiers() -> ModifierState {
        ModifierState::new()
    }

    /// Create left ctrl modifier state
    pub fn left_ctrl() -> ModifierState {
        let mut mods = ModifierState::new();
        mods.left_ctrl = true;
        mods
    }

    /// Create left shift modifier state
    pub fn left_shift() -> ModifierState {
        let mut mods = ModifierState::new();
        mods.left_shift = true;
        mods
    }

    /// Create left alt modifier state
    pub fn left_alt() -> ModifierState {
        let mut mods = ModifierState::new();
        mods.left_alt = true;
        mods
    }

    /// Add simple key press mapping
    pub fn map_key_press(
        &mut self,
        keycode: Keycode,
        modifiers: ModifierState,
        commands: Vec<Commands>,
    ) {
        let mapping = KeyMapping {
            pattern: KeyPattern::Single(KeyWithModifiers { keycode, modifiers }),
            trigger: TriggerType::OnPress,
            press_commands: commands,
            release_commands: vec![],
            timeout_ms: None,
        };
        self.add_mapping(mapping);
    }

    /// Add simple key release mapping
    pub fn map_key_release(
        &mut self,
        keycode: Keycode,
        modifiers: ModifierState,
        commands: Vec<Commands>,
    ) {
        let mapping = KeyMapping {
            pattern: KeyPattern::Single(KeyWithModifiers { keycode, modifiers }),
            trigger: TriggerType::OnRelease,
            press_commands: vec![],
            release_commands: commands,
            timeout_ms: None,
        };
        self.add_mapping(mapping);
    }

    /// Add chord mapping
    pub fn map_chord(&mut self, keys: Vec<(Keycode, ModifierState)>, commands: Vec<Commands>) {
        let chord_keys: Vec<KeyWithModifiers> = keys
            .into_iter()
            .map(|(keycode, modifiers)| KeyWithModifiers { keycode, modifiers })
            .collect();

        let mapping = KeyMapping {
            pattern: KeyPattern::Chord(chord_keys),
            trigger: TriggerType::OnPress,
            press_commands: commands,
            release_commands: vec![],
            timeout_ms: None,
        };
        self.add_mapping(mapping);
    }

    /// Add sequence mapping
    pub fn map_sequence(
        &mut self,
        keys: Vec<(Keycode, ModifierState)>,
        commands: Vec<Commands>,
        timeout_ms: Option<u64>,
    ) {
        let sequence_keys: Vec<KeyWithModifiers> = keys
            .into_iter()
            .map(|(keycode, modifiers)| KeyWithModifiers { keycode, modifiers })
            .collect();

        let mapping = KeyMapping {
            pattern: KeyPattern::Sequence(sequence_keys),
            trigger: TriggerType::OnPress,
            press_commands: commands,
            release_commands: vec![],
            timeout_ms,
        };
        self.add_mapping(mapping);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn test_key_mapper_creation() {
        let (tx, _rx) = mpsc::channel();
        let mapper = KeyMapper::new(tx);
        assert_eq!(mapper.press_mappings.len(), 0);
        assert_eq!(mapper.chord_mappings.len(), 0);
    }

    #[test]
    fn test_modifier_state() {
        let mods = ModifierState::new();
        assert!(!mods.left_ctrl);

        let ctrl_mods = KeyMapper::left_ctrl();
        assert!(ctrl_mods.left_ctrl);
        assert!(!ctrl_mods.right_ctrl);
    }
}
