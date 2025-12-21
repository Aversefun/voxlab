//! Entrypoint for voxlab.

mod audio;
mod dsp;
mod nice;
mod phoneme;

use std::error::Error;

use crate::{
    audio::wav,
    dsp::{pitch::pitch_shift, stretch::time_stretch},
    phoneme::ipa::Vowel,
};

fn main() -> Result<(), Box<dyn Error>> {
    let file = wav::import_wav(format!(
        "samples/vowel_{}.wav",
        Vowel::OpenBackUnrounded.ipa()
    ))?;
    println!("Original: {} samples ({} sample rate)", file.samples.len(), file.sample_rate);

    for ratio in [1.0f32, 0.5, 2.0] {
        let buf = time_stretch(&pitch_shift(&file, ratio), ratio);
        println!("{ratio}x pitch and time: {} samples ({} sample rate)", buf.samples.len(), buf.sample_rate);
        wav::export_wav(
            buf,
            format!("output_{}.wav", ratio),
        )?;
    }

    Ok(())
}
