//! Entrypoint for voxlab.

mod audio;
mod dsp;
mod nice;
mod phoneme;
mod plotting;

use std::error::Error;

use plotters::style::{BLACK, full_palette::PURPLE};

use crate::{
    audio::{buffer::AudioBuffer, wav::{self, export_wav}},
    dsp::{
        crossfade::{self, crossfade},
        psola::psola_constant,
    },
    phoneme::ipa::Vowel,
    plotting::Plot,
};

fn render_single(vowel: Vowel, ratio: f32) -> Result<AudioBuffer, Box<dyn Error>> {
    let file = wav::import_wav(format!("samples/vowel_{}.wav", vowel.ipa()))?;
    println!(
        "Original: {} samples ({} sample rate)",
        file.samples.len(),
        file.sample_rate
    );

    let file_name = format!("plot_{}_{}.png", ratio, vowel.ipa());

    let mut plot = Plot::new(&file_name, "Glide", 0.0..1.0, 0.0..4.0, 0.05)?;

    plot.plot(
        |x| file.samples[(x * file.samples.len() as f32).floor() as usize] * 4.0,
        PURPLE,
        "Input audio",
    )?;

    // const VIBRATO_DEPTH: f32 = 1.1;
    // const VIBRATO_RATE: f32 = 0.65;

    // let buf = time_glide(
    //     &pitch_glide(
    //         &file,
    //         |t| ratio * (1.0 + VIBRATO_DEPTH * (TAU * VIBRATO_RATE * t).sin()),
    //         &mut plot,
    //     ),
    //     |t| 1.0 + 0.05 * (TAU * t).sin(),
    //     &mut plot,
    // );

    let buf = psola_constant(&file, ratio, 1.0 / ratio, &mut plot);
    plot.plot(
        |x| buf.samples[(x * buf.samples.len() as f32).floor() as usize] * 4.0,
        BLACK,
        "Output audio",
    )?;

    println!(
        "{ratio}x pitch and time: {} samples ({} sample rate)",
        buf.samples.len(),
        buf.sample_rate
    );

    Ok(buf)
}

fn main() -> Result<(), Box<dyn Error>> {
    for ratio in [0.25f32, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0] {
        let vowel1 = render_single(Vowel::OpenBackUnrounded, ratio)?;
        let vowel2 = render_single(Vowel::CloseFrontUnrounded, ratio)?;

        let out = crossfade(&vowel1, &vowel2, crossfade::CROSSFADE_TIME);

        export_wav(out, format!("output_{ratio}.wav"));
    }

    Ok(())
}
