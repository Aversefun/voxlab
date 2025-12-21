//! Entrypoint for voxlab.

mod audio;
mod dsp;
mod nice;
mod phoneme;
mod plotting;

use std::{error::Error, f32::consts::TAU};

use crate::{
    audio::wav,
    dsp::{pitch::pitch_shift, stretch::time_glide},
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

    for ratio in [1.0f32, 0.5, 2.0, 0.25, 1.25] {
        let file_name = format!("plot_{}.png", ratio);

        let mut plot = Plot::new(&file_name, "Time glide", 0.0..1.0, 0.0..2.0, 0.1)?;

        let buf = time_glide(
            &pitch_shift(&file, ratio),
            |t| 1.0 + (ratio * (TAU * t).sin()).abs(),
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
