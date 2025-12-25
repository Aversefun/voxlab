//! Sample loading.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    audio::{MidiNote, buffer::AudioBuffer, wav},
    dsp::{
        crossfade::find_voiced_region,
        psola::{generate_pitch_marks, get_avg_period},
        window_calc::find_window,
    },
    phoneme::ipa::Phoneme,
};

pub use hound::Result;

macro_rules! cached_func {
    ($(#[$meta:meta])* $name:ident ($prop:ident) -> $ty:ty => $calc:expr) => {
        $(#[$meta])*
        pub fn $name (&mut self, phoneme: Phoneme) -> hound::Result<&$ty> {
            if !self.$prop.contains_key(&phoneme) {
                let res = $calc(self, phoneme)?;
                self.$prop.insert(phoneme, res);
            }
            Ok(&self.$prop[&phoneme])
        }
    };
    ($(#[$meta:meta])* $name:ident -> $ty:ty => $calc:expr) => {
        cached_func!($(#[$meta])* $name ($name) -> $ty => $calc);
    };
}

/// Will eventually be populated with options.
#[derive(Clone, Debug)]
pub struct Voice {
    root: PathBuf,
    cache: HashMap<Phoneme, AudioBuffer>,
    sample_rate: u32,
    pitches: HashMap<Phoneme, MidiNote>,
    pitch_marks: HashMap<Phoneme, Vec<usize>>,
}

impl Voice {
    pub fn new(
        root: impl AsRef<Path>,
        sample_rate: u32,
        pitches: HashMap<Phoneme, MidiNote>,
    ) -> Self {
        Self {
            root: root.as_ref().to_owned(),
            cache: HashMap::new(),
            sample_rate,
            pitches,
            pitch_marks: HashMap::new(),
        }
    }
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    cached_func!(
        /// Returns the MIDI note number of the specified phoneme, estimating if no
        /// known note.
        base_note (pitches) -> MidiNote => |this: &mut Self, phoneme: Phoneme| -> Result<_> {
            Ok(MidiNote({
                let sample = this.sample(phoneme).unwrap();
                let voiced_region = find_voiced_region(sample).unwrap_or((0, sample.len()));
                let sample = AudioBuffer {
                    sample_rate: sample.sample_rate,
                    samples: sample.samples[voiced_region.0..voiced_region.1].to_vec(),
                };

                let avg_period = dbg!(get_avg_period(&sample));
                let f0 = dbg!(sample.sample_rate as f32 / avg_period as f32);
                dbg!(69.0 + 12.0 * (f0 / 440.0).log2())
            }))
        }
    );

    cached_func!(
        pitch_marks -> [usize] => |this: &mut Self, phoneme: Phoneme| -> Result<_> {
            Ok({
                let sample = this.sample(phoneme).unwrap();

                let windows = find_window(&sample, None);
                generate_pitch_marks(&sample, &windows)
            })
        }
    );

    cached_func!(
        sample (cache) -> AudioBuffer => |this: &mut Self, phoneme: Phoneme| -> Result<_> {
            Ok(match phoneme {
                Phoneme::Space => AudioBuffer {
                    sample_rate: this.sample_rate(),
                    samples: vec![0.0; 256],
                },
                Phoneme::Vowel(_) => {
                    wav::import_wav(this.root.join(format!("vowel_{}.wav", phoneme.ipa())))?
                }
                Phoneme::Consonant(_) => {
                    wav::import_wav(this.root.join(format!("consonant_{}.wav", phoneme.ipa())))?
                }
            })
        }
    );
}
