//! Audio storage.

/// Mono audio buffer.
#[derive(Clone, Debug, PartialEq)]
pub struct AudioBuffer {
    pub sample_rate: u32,
    pub samples: Vec<f32>,
}

impl AudioBuffer {
    pub fn len(&self) -> usize {
        self.samples.len()
    }
}
