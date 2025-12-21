//! Audio storage.

/// Mono audio buffer.
#[derive(Clone, Debug, PartialEq)]
pub struct AudioBuffer {
    pub sample_rate: u32,
    pub samples: Vec<f32>,
}

impl AudioBuffer {
    /// Map one audio buffer to another.
    pub fn map(&self, f: impl FnMut(f32) -> f32) -> Self {
        Self {
            sample_rate: self.sample_rate,
            samples: self.samples.iter().copied().map(f).collect()
        }
    }
}