#[derive(Debug)]
pub enum KeyType {
    Note,
    Modifier,
    Control,
}

/// All the keys present in the program.
/// Every KEY corresponds to an action.
/// They can either be up/down like the note keys or one-shot like the change octave_key.
#[derive(Debug)]
pub enum KeyCode {
    /// Notes
    NoteC,
    NoteCS,
    NoteD,
    NoteDS,
    NoteE,
    NoteF,
    NoteFS,
    NoteG,
    NoteGS,
    NoteA,
    NoteAS,
    NoteB,
    NoteCUp,
    NoteCSUp,
    NoteDUp,
    NoteDSUp,
    NoteEUp,
    NoteFUp,
    NoteFSUp,
    NoteGUp,
    NoteGSUp,
    NoteAUp,
    NoteASUp,
    NoteBUp,
    /// Basic controls
    OctaveUp,
    OctaveDown,
    /// Modifiers
    ModOctaveUp,
    ModOctaveDown,
}

/// Represents a Key for the program
/// This is the structure that is used by the core to perform actions.
/// Different input methods can map to these to interact with the project.
#[derive(Debug)]
pub struct Key {
    /// The code of the key
    pub code: KeyCode,
    pub ktype: KeyType,
    /// Wether the keys has a sustain effect (e.g. notes can be kept playing)
    pub sustain: bool,
}
