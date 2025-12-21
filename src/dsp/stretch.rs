//! Time-stretch.

use crate::{audio::buffer::AudioBuffer, dsp::window::hann};

const WINDOW_SIZE: usize = 1024;
const ANALYSIS_HOP: usize = 256;

/// Naive OLA time-stretch. Robotic artifacts expected (it does not sound human at all lol).
pub fn time_stretch(input: &AudioBuffer, stretch: f32) -> AudioBuffer {
    let output_length = (input.samples.len() as f32 * stretch).ceil() as usize;
    let synthesis_hop = (ANALYSIS_HOP as f32 * stretch).floor() as usize;

    let window = hann(WINDOW_SIZE);

    let mut output = vec![0.0f32; output_length];

    let mut in_pos = 0usize;
    let mut out_pos = 0usize;

    while (in_pos + WINDOW_SIZE) < input.samples.len() {
        let mut frame = input.samples[in_pos..(in_pos + WINDOW_SIZE)].to_vec();

        for (i, offset) in window.iter().copied().enumerate() {
            frame[i] *= offset;
        }

        for (i, out) in frame.iter().copied().enumerate() {
            if out_pos + i >= output.len() {
                break;
            }
            output[out_pos + i] += out;
        }

        in_pos += ANALYSIS_HOP;
        out_pos += synthesis_hop;
    }

    AudioBuffer {
        sample_rate: input.sample_rate,
        samples: output,
    }
}
