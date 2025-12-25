//! Entrypoint for voxlab.

mod audio;
mod dsp;
mod nice;
mod phoneme;
mod plotting;
mod samples;
mod scheduling;

use std::{collections::HashMap, error::Error};

use crate::{
    audio::{MidiNote, wav::export_wav},
    phoneme::ipa::{Phoneme, Vowel},
    samples::Voice,
    scheduling::{InstanceId, PhonemeInstance, PhonemeOptions, Schedule as _, TransitionOptions},
};

fn main() -> Result<(), Box<dyn Error>> {
    let phonemes = (50..=60usize)
        .map(|i| PhonemeInstance {
            instance_id: InstanceId::new(i),
            steady_grains: 20,
            phoneme: Phoneme::Vowel(Vowel::CloseFrontUnrounded),
            length: 1.0,
            options: PhonemeOptions {
                next_transition: Some(TransitionOptions { length_grains: 10 }),
            },
            note: MidiNote(i as f32),
        })
        .collect::<Vec<_>>();

    let scheduled = dbg!(phonemes.schedule());

    let rendered = scheduled.render(&mut Voice::new("samples", 44100, HashMap::new()))?;

    export_wav(rendered, "outputs/5d/output.wav")?;

    Ok(())
}
