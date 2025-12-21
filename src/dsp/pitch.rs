//! Pitch adjustment.

use crate::{audio::buffer::AudioBuffer, nice::lerp};

/// Naive pitch shift.
pub fn pitch_shift(input: &AudioBuffer, ratio: f32) -> AudioBuffer {
    assert!(ratio > 0.0);

    let in_len = input.samples.len();
    let out_len = (in_len as f32 / ratio).floor() as usize;

    let mut out = Vec::with_capacity(out_len);

    for i in 0..out_len {
        let pos = i as f32 * ratio;
        let idx = pos.floor() as usize;
        let frac = pos - idx as f32;

        let s0 = input.samples[idx];
        let s1 = input.samples.get(idx + 1).copied().unwrap_or(s0);

        out.push(lerp(s0, s1, frac));
    }

    AudioBuffer {
        sample_rate: input.sample_rate,
        samples: out,
    }
}
