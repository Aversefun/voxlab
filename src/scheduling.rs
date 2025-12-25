//! Scheduling multiple phonemes.

use std::fmt::{Debug, Display};

use crate::{
    audio::{MidiNote, buffer::AudioBuffer},
    dsp::{
        crossfade::{GrainInterp, crossfade},
        psola::psola_constant,
    },
    phoneme::ipa::Phoneme,
    samples::{self, Voice},
};

/// The canonical ID of a specific [`PhonemeInstance`]. Should be mostly treated
/// opaquely, but is an index internally.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct InstanceId(usize);

impl InstanceId {
    pub fn new(index: usize) -> Self {
        Self(index)
    }
    pub fn index(self) -> usize {
        self.0
    }
}

impl Debug for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("P")?;
        Debug::fmt(&self.0, f)
    }
}

impl Display for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhonemeOptions {
    pub next_transition: Option<TransitionOptions>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransitionOptions {
    pub length_grains: usize,
}

#[derive(Clone, PartialEq)]
pub struct PhonemeInstance {
    pub instance_id: InstanceId,
    pub steady_grains: usize,
    pub phoneme: Phoneme,
    /// Length multiplier.
    pub length: f32,
    pub options: PhonemeOptions,
    pub note: MidiNote,
}

impl Debug for PhonemeInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhonemeInstance")
            .field("instance_id", &self.instance_id)
            .field("phoneme", &self.phoneme)
            .field("next_transition", &self.options.next_transition)
            .field("steady_grains", &self.steady_grains)
            .field("pitch", &self.note)
            .field("length", &self.length)
            .finish()
    }
}

/// One scheduled grain in the output timeline.
#[derive(Clone, Debug)]
pub struct GrainEvent {
    /// Identity of the phoneme instance that produced this grain
    pub instance_id: InstanceId,
    /// Source phoneme to read from
    pub source: Phoneme,
    /// How long to play this for. Multiplier.
    pub length: f32,
    /// The note that this phoneme should be played at.
    pub note: MidiNote,
    /// Optional interpolation to a target phoneme
    pub interp: Option<GrainInterp>,
}

/// A fully-resolved, linear plan for grain-based synthesis.
/// Voice-agnostic.
#[derive(Clone, Debug)]
pub struct GrainTimeline {
    pub events: Vec<GrainEvent>,
}

impl GrainTimeline {
    pub fn render(&self, voice: &mut Voice) -> samples::Result<AudioBuffer> {
        let mut out = Vec::new();
        let mut last: Option<(&GrainEvent, Vec<f32>)> = None;
        for event in &self.events {
            let base_note = dbg!(*voice.base_note(event.source)?);

            let buf = voice.sample(event.source)?;

            let semitone_diff = dbg!(event.note.0 - base_note.0);
            let pitch_ratio = 2.0_f32.powf(semitone_diff / 12.0);

            let cur = psola_constant(buf, dbg!(pitch_ratio), dbg!(event.length), None).samples;

            let temp;

            out.extend(
                if let Some(last) = last
                    && let Some(interp) = &last.0.interp
                {
                    temp = crossfade(
                        &AudioBuffer {
                            sample_rate: voice.sample_rate(),
                            samples: last.1,
                        },
                        &AudioBuffer {
                            sample_rate: voice.sample_rate(),
                            samples: cur.clone(),
                        },
                        interp,
                    )
                    .samples;
                    &temp
                } else {
                    &cur
                },
            );

            eprintln!("rendered event {}", event.instance_id);

            last = Some((event, cur));
        }

        Ok(AudioBuffer {
            sample_rate: voice.sample_rate(),
            samples: out,
        })
    }
}

trait PrepareSealed {}

#[expect(private_bounds, reason = "intended")]
pub trait Schedule: PrepareSealed {
    fn schedule(&self) -> GrainTimeline;
}

impl<T: AsRef<[PhonemeInstance]>> PrepareSealed for T {}

impl<T: AsRef<[PhonemeInstance]>> Schedule for T {
    fn schedule(&self) -> GrainTimeline {
        let phonemes = self.as_ref();

        let mut out = Vec::new();
        for (i, phoneme) in phonemes.iter().enumerate() {
            let next = phonemes.get(i + 1);

            if let Some(trans) = &phoneme.options.next_transition
                && next.is_some()
            {
                debug_assert!(
                    trans.length_grains <= phoneme.steady_grains,
                    "transition longer than steady region"
                );

                let interp = GrainInterp {
                    target_grain: i,
                    fade_len: trans.length_grains,
                };

                out.push(GrainEvent {
                    instance_id: phoneme.instance_id,
                    source: phoneme.phoneme,
                    length: phoneme.length,
                    note: phoneme.note,
                    interp: Some(interp),
                });
            } else {
                out.push(GrainEvent {
                    instance_id: phoneme.instance_id,
                    note: phoneme.note,
                    source: phoneme.phoneme,
                    length: phoneme.length,
                    interp: None,
                });
            }
        }

        GrainTimeline { events: out }
    }
}
