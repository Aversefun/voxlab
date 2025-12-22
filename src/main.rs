//! Entrypoint for voxlab.

mod audio;
mod dsp;
mod nice;
mod phoneme;
mod plotting;

use std::{error::Error, f32::consts::TAU};

use plotters::style::{
    BLACK,
    full_palette::PURPLE,
};

use crate::{
    audio::wav,
    dsp::{
        pitch::pitch_glide,
        stretch::time_glide, window_calc::find_window,
    },
    phoneme::ipa::Vowel,
    plotting::Plot,
};

fn main() -> Result<(), Box<dyn Error>> {
    let file = wav::import_wav(format!(
        "samples/vowel_{}.wav",
        Vowel::OpenBackUnrounded.ipa()
    ))?;
    println!(
        "Original: {} samples ({} sample rate)",
        file.samples.len(),
        file.sample_rate
    );

    for ratio in [1.0f32, 0.5, 2.0, 0.25, 1.25, 1.5, 0.75] {
        let file_name = format!("plot_{}.png", ratio);

        let mut plot = Plot::new(&file_name, "Glide", 0.0..1.0, 0.0..4.0, 0.05)?;

        plot.plot(
            |x| file.samples[(x * file.samples.len() as f32).floor() as usize] * 4.0,
            PURPLE,
            "Input audio",
        )?;

        find_window(&file, &mut plot);

        const VIBRATO_DEPTH: f32 = 1.1;
        const VIBRATO_RATE: f32 = 0.65;

        let buf = time_glide(
            &pitch_glide(
                &file,
                |t| ratio * (1.0 + VIBRATO_DEPTH * (TAU * VIBRATO_RATE * t).sin()),
                &mut plot,
            ),
            |t| 1.0 + 0.05 * (TAU * t).sin(),
            &mut plot,
        );
        // plot.plot(
        //     |x| buf.samples[(x * buf.samples.len() as f32).floor() as usize] * 4.0,
        //     BLACK,
        //     "Output audio",
        // )?;

        println!(
            "{ratio}x pitch and time: {} samples ({} sample rate)",
            buf.samples.len(),
            buf.sample_rate
        );
        wav::export_wav(buf, format!("output_{}.wav", ratio))?;
    }

    Ok(())
}
