//! Time-stretch.

use plotters::style::{BLUE, RED};

use crate::{
    audio::buffer::AudioBuffer,
    dsp::{low_pass, window::hann},
    plotting::Plot,
};

const WINDOW_SIZE: usize = 1024;
const ANALYSIS_HOP: usize = 256;

/// Naive OLA time-stretch. Robotic artifacts expected (it does not sound human
/// at all lol).
#[deprecated = "use PSOLA"]
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

/// Time-stretch where it changes over time. The function takes in a value
/// between 0 and 1 (where 0 is the first frame and 1 is the last) and outputs
/// how much to stretch.
#[deprecated = "use PSOLA"]
pub fn time_glide(
    input: &AudioBuffer,
    mut stretch: impl FnMut(f32) -> f32,
    plot: &mut Plot<'_>,
) -> AudioBuffer {
    plot.plot(&mut stretch, &BLUE, "raw time glide").unwrap();

    let window = hann(WINDOW_SIZE);

    let mut output = vec![0.0f32; input.samples.len()];

    let mut in_pos = 0usize;
    let mut in_frame = 0usize;
    let mut out_pos = 0usize;

    let mut stretch_vals = Vec::new();

    let total_frames = input.samples.len() / ANALYSIS_HOP;

    for i in 0..total_frames {
        let t = i as f32 / total_frames as f32;
        stretch_vals.push(stretch(t));
    }

    stretch_vals = low_pass(stretch_vals, 0.05);

    plot.plot(
        |x| stretch_vals[(x * total_frames as f32) as usize],
        &RED,
        "filtered time glide",
    )
    .unwrap();

    while (in_pos + WINDOW_SIZE) < input.samples.len() {
        let stretch = stretch_vals[in_frame];
        assert!(stretch > 0.0);

        let synthesis_hop = (ANALYSIS_HOP as f32 * stretch) as usize;

        let mut frame = input.samples[in_pos..(in_pos + WINDOW_SIZE)].to_vec();

        for (i, offset) in window.iter().copied().enumerate() {
            frame[i] *= offset;
        }

        for (i, out) in frame.iter().copied().enumerate() {
            if out_pos + i >= input.samples.len() {
                output.push(0.0f32);
            }
            output[out_pos + i] += out;
        }

        in_pos += ANALYSIS_HOP;
        out_pos += synthesis_hop;
        in_frame += 1;
    }

    AudioBuffer {
        sample_rate: input.sample_rate,
        samples: output,
    }
}
