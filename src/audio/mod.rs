//! Audio importing and storage.

pub mod buffer;
pub mod wav;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MidiNote(pub f32);
