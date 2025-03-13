// The mod score contains all the building block for creating music
// Sheets contain instruments layed out on a staff, divided into measures

mod measure;
mod score;
mod staff;

pub mod prelude {
    pub use super::score::*;
}
