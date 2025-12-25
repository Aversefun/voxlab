//! Calculations for finding a good window for PSOLA.

use plotters::style::{BLACK, full_palette::PURPLE};

use crate::{audio::buffer::AudioBuffer, plotting::Plot};

const ANALYSIS_WINDOW: usize = 1024;

pub fn find_window_single(buffer: &[f32; ANALYSIS_WINDOW], sample_rate: u32) -> usize {
    let min_lag = sample_rate as usize / 300;
    let max_lag = (sample_rate as usize / 80).min(ANALYSIS_WINDOW - 1);

    let mut best_score = f32::NEG_INFINITY;
    let mut best_lag = min_lag;

    for lag in min_lag..=max_lag {
        let mut score = 0.0f32;

        for i in 0..(ANALYSIS_WINDOW - lag) {
            score += buffer[i] * buffer[i + lag];
        }

        score /= (ANALYSIS_WINDOW - lag) as f32;

        if score > best_score {
            best_score = score;
            best_lag = lag;
        }
    }

    best_lag
}

const HOP: usize = ANALYSIS_WINDOW / 4;

pub fn find_window(buffer: &AudioBuffer, plot: Option<&mut Plot>) -> Vec<(usize, usize)> {
    let mut results = Vec::new();
    let mut start = 0usize;

    while start + ANALYSIS_WINDOW <= buffer.samples.len() {
        let frame = buffer.samples[start..(start + ANALYSIS_WINDOW)]
            .as_array()
            .unwrap();
        let lag = find_window_single(frame, buffer.sample_rate);

        results.push((start, lag));

        start += HOP;
    }

    plot.map(|plot| {
        plot.plot_points(
            |x| results[(x * results.len() as f32) as usize].0 as f32 / buffer.samples.len() as f32,
            &PURPLE,
            "Window starts",
        )
        .unwrap();

        plot.plot_points(
            |x| results[(x * results.len() as f32) as usize].1 as f32 / ANALYSIS_WINDOW as f32,
            &BLACK,
            "Window lengths",
        )
        .unwrap();
    });

    results
}
