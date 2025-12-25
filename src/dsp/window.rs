//! Audio windows.

use std::f32::consts::PI;

pub fn hann(len: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; len];

    for n in 0..len {
        out[n] = 0.5 * (1.0 - (2.0 * PI * n as f32 / ((len - 1) as f32)).cos())
    }

    return out;
}

fn apply_window(samples: &[f32], window: &[f32], out: &mut [f32]) {
    for i in 0..samples.len() {
        out[i] += samples[i] * window[i];
    }
}
