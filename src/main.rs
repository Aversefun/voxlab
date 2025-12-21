//! Entrypoint for voxlab.

mod audio;
mod phoneme;
mod dsp;
mod nice;

use std::error::Error;

use crate::{audio::wav, dsp::pitch::pitch_shift, phoneme::ipa::Vowel};

fn main() -> Result<(), Box<dyn Error>> {
    let file = wav::import_wav(format!("samples/vowel_{}.wav", Vowel::OpenBackUnrounded.ipa()))?;
    
    for ratio in [1.0f32, 0.5, 2.0] {
        wav::export_wav(pitch_shift(&file, ratio), format!("output_{}.wav", ratio))?;
    }

    Ok(())
}
