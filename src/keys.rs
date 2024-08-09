/// All the keys present in the program.
/// Every KEY corresponds to an action.
/// They can either be up/down like the note keys or one-shot like the change octave_key.
pub enum KEYS {
    /// Notes
    NOTE_C,
    NOTE_CS,
    NOTE_D,
    NOTE_DS,
    NOTE_E,
    NOTE_F,
    NOTE_FS,
    NOTE_G,
    NOTE_GS,
    NOTE_A,
    NOTE_AS,
    NOTE_B,
    NOTE_C_UP,
    NOTE_CS_UP,
    NOTE_D_UP,
    NOTE_DS_UP,
    NOTE_E_UP,
    NOTE_F_UP,
    NOTE_FS_UP,
    NOTE_G_UP,
    NOTE_GS_UP,
    NOTE_A_UP,
    NOTE_AS_UP,
    NOTE_B_UP,
    /// Basic controls
    OCTAVE_UP,
    OCTAVE_DOWN,
    /// Modifiers
    MOD_OCTAVE_UP,
    MOD_OCTAVE_DOWN,
}
