//! Entrypoint for voxlab.

mod audio;
mod dsp;
mod nice;
mod phoneme;
mod plotting;

use std::{error::Error, f32::consts::TAU};

use crate::{
    audio::wav,
    dsp::{
        pitch::{pitch_glide, pitch_shift},
        stretch::time_glide,
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
        println!(
            "{ratio}x pitch and time: {} samples ({} sample rate)",
            buf.samples.len(),
            buf.sample_rate
        );
        wav::export_wav(buf, format!("output_{}.wav", ratio))?;
    }

    Ok(())
}
