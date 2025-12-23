//! PSOLA adjustment.

use crate::{
    audio::buffer::AudioBuffer,
    dsp::{window::hann, window_calc::find_window},
    plotting::Plot,
};

pub fn generate_pitch_marks(buffer: &AudioBuffer, windows: &[(usize, usize)]) -> Vec<usize> {
    let mut marks = Vec::new();

    if windows.is_empty() {
        return marks;
    }

    let samples_len = buffer.samples.len();

    let mut pos = windows[0].0;
    let mut window_idx = 0;
    let mut current_lag = windows[0].1;

    while pos < samples_len {
        marks.push(pos);

        while window_idx + 1 < windows.len() && windows[window_idx + 1].0 <= pos {
            window_idx += 1;
            current_lag = windows[window_idx].1;
        }

        pos += current_lag;

        if current_lag == 0 {
            break;
        }
    }

    marks
}

/// Return index of first element > bound.
pub fn upper_bound(marks: &[usize], bound: usize) -> usize {
    marks.partition_point(|&m| m <= bound)
}

/// Last mark <= bound.
pub fn last_mark_leq(marks: &[usize], bound: usize) -> Option<usize> {
    let i = upper_bound(marks, bound);
    if i == 0 { None } else { Some(marks[i - 1]) }
}

/// First mark >= bound.
pub fn first_mark_geq(marks: &[usize], bound: usize) -> Option<usize> {
    let i = marks.partition_point(|&m| m < bound);
    marks.get(i).copied()
}

pub struct Grain<'a> {
    center: usize,
    period: usize,
    samples: &'a [f32],
}

pub fn lag_at(pos: usize, windows: &[(usize, usize)]) -> usize {
    let mut lag = windows[0].1;
    for &(wpos, wlag) in windows {
        if wpos <= pos {
            lag = wlag;
        } else {
            break;
        }
    }
    lag
}

pub fn extract_grains<'a>(
    buffer: &'a AudioBuffer,
    marks: &[usize],
    windows: &[(usize, usize)],
) -> Vec<Grain<'a>> {
    let mut grains = Vec::new();
    let samples = &buffer.samples;
    let len = samples.len();

    for &mark in marks {
        let period = lag_at(mark, windows);
        let half = period;

        if mark < half || mark + half >= len {
            continue;
        }

        let start = mark - half;
        let end = mark + half;

        let grain_samples = &samples[start..end];

        grains.push(Grain {
            center: mark,
            period,
            samples: grain_samples,
        });
    }

    grains
}

pub fn get_avg_period(input: &AudioBuffer) -> usize {
    let windows = find_window(input, None);
    let marks = generate_pitch_marks(input, &windows);
    let grains = extract_grains(input, &marks, &windows);

    grains.iter().map(|g| g.period).sum::<usize>() / grains.len()
}

pub fn psola_constant(
    input: &AudioBuffer,
    pitch_ratio: f32,
    time_stretch: f32,
    plot: &mut Plot,
) -> AudioBuffer {
    let analysis_windows = &find_window(input, Some(plot));

    assert!(pitch_ratio > 0.0);
    assert!(time_stretch > 0.0);
    if input.samples.len() < 2048 || analysis_windows.is_empty() {
        return input.clone();
    }

    let marks = generate_pitch_marks(input, analysis_windows);

    let grains = extract_grains(input, &marks, analysis_windows);
    if grains.is_empty() {
        return input.clone();
    }

    let in_len = input.samples.len();
    let mut out_len = (in_len as f32 * time_stretch).ceil() as usize;
    out_len = out_len.saturating_add(4096);

    let mut out = vec![0.0f32; out_len];
    let mut overlap_count = vec![0.0f32; out_len];

    let mut out_center_f = grains[0].period as f32;
    for (gi, grain) in grains.iter().enumerate() {
        // let t = if grains.len() <= 1 {
        //     0.0
        // } else {
        //     gi as f32 / (grains.len() - 1) as f32
        // };

        let r = pitch_ratio;
        let s = time_stretch;

        let new_period = (grain.period as f32 / r).max(1.0);
        let grain_len = grain.samples.len();

        let w = hann(grain_len);

        let out_center = out_center_f.round() as isize;
        let half = (grain_len / 2) as isize;
        let start = out_center - half;

        for i in 0..grain_len {
            let out_i = start + i as isize;
            if out_i < 0 || out_i as usize >= out.len() {
                continue;
            }
            let idx = out_i as usize;
            let v = grain.samples[i] * w[i];
            out[idx] += v;
            overlap_count[idx] += w[i];
        }

        out_center_f += new_period * s;

        if out_center_f as usize >= out.len().saturating_sub(grain_len + 1) {
            break;
        }
    }

    for i in 0..out.len() {
        let d = overlap_count[i];
        if d > 1e-6 {
            out[i] /= d;
        }
    }

    let mut trim = out.len();
    while trim > 0 && out[trim - 1].abs() < 1e-6 {
        trim -= 1;
    }
    out.truncate(trim.max(1));

    let mut peak = 0.0f32;
    for &s in &out {
        peak = peak.max(s.abs());
    }
    if peak > 1.0 {
        for s in &mut out {
            *s /= peak;
        }
    }

    AudioBuffer {
        sample_rate: input.sample_rate,
        samples: out,
    }
}
