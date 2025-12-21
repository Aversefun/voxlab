//! Importing wav files.

use std::path::Path;

use hound::WavSpec;

use crate::audio::buffer::AudioBuffer;

/// Import a mono WAV file and convert it to a normalized `AudioBuffer`.
///
/// Supported formats:
/// - 16-bit PCM
/// - 32-bit PCM
/// - 32-bit float
///
/// Stereo files are rejected.
pub fn import_wav(path: impl AsRef<Path>) -> hound::Result<AudioBuffer> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();

    if spec.channels != 1 {
        panic!("only mono WAV files are supported");
    }

    let samples = match spec.sample_format {
        hound::SampleFormat::Float => {
            reader
                .samples::<f32>()
                .collect::<Result<Vec<_>, _>>()?
        }

        hound::SampleFormat::Int => {
            let max = (1i64 << (spec.bits_per_sample - 1)) as f32;

            reader
                .samples::<i32>()
                .map(|s| s.map(|v| v as f32 / max))
                .collect::<Result<Vec<_>, _>>()?
        }
    };

    Ok(AudioBuffer {
        sample_rate: spec.sample_rate,
        samples,
    })
}

/// Export a WAV file.
pub fn export_wav(buffer: AudioBuffer, path: impl AsRef<Path>) -> hound::Result<()> {
    let mut writer = hound::WavWriter::create(path, WavSpec {
        channels: 1,
        sample_rate: buffer.sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float
    })?;
    for sample in buffer.samples {
        writer.write_sample(sample)?;
    }
    writer.finalize()?;
    Ok(())
}

