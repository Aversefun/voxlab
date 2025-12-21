//! Pitch adjustment.

use plotters::style::{GREEN, full_palette::ORANGE};

use crate::{audio::buffer::AudioBuffer, dsp::low_pass, nice::lerp, plotting::Plot};

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

pub fn pitch_glide(
    input: &AudioBuffer,
    mut ratio: impl FnMut(f32) -> f32,
    plot: &mut Plot<'_>,
) -> AudioBuffer {
    let in_len = input.samples.len();

    let mut out = Vec::with_capacity(in_len);

    let mut i = 0.0f32;

    plot.plot(&mut ratio, &GREEN, "raw pitch glide").unwrap();

    let mut pitch_vals = Vec::with_capacity(in_len);

    while i < in_len as f32 {
        let ratio = ratio(i / in_len as f32);
        pitch_vals.push(ratio);
        i += ratio * 0.5;
    }
    i = 0.0;

    pitch_vals = low_pass(pitch_vals, 0.2);

    plot.plot(
        |x| pitch_vals[(x * pitch_vals.len() as f32) as usize],
        &ORANGE,
        "filtered pitch glide",
    )
    .unwrap();

    let mut pitch_i = 0usize;
    while i < in_len as f32 {
        let ratio = pitch_vals[pitch_i];
        // println!("{i} increasing by {ratio}");
        assert!(ratio > 0.0);
        i += ratio;
        pitch_i += 1;

        let idx = i.floor() as usize;
        let frac = i - idx as f32;

        if idx >= input.samples.len() {
            break;
        }

        let s0 = input.samples[idx];
        let s1 = input.samples.get(idx + 1).copied().unwrap_or(s0);

        out.push(lerp(s0, s1, frac));
    }

    AudioBuffer {
        sample_rate: input.sample_rate,
        samples: out,
    }
}
