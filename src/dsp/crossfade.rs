//! Crossfade between two buffers.

use std::f32::consts::FRAC_PI_2;

use crate::{
    audio::buffer::AudioBuffer,
    dsp::{
        psola::{self, get_avg_period},
        window_calc::find_window,
    },
};

pub const CROSSFADE_TIME: usize = 2205;

const MIN_TRANSITION_PERIODS: usize = 5;

pub fn crossfade(buf1: &AudioBuffer, buf2: &AudioBuffer, fade_len: usize) -> AudioBuffer {
    let period = get_avg_period(buf1);

    let windows_a = find_window(buf1, None);
    let marks_a = psola::generate_pitch_marks(buf1, &windows_a);

    let (voiced_start_a, voiced_end_a) = find_voiced_region(&buf1).unwrap();
    let cut_a = find_cut_a(
        &marks_a,
        voiced_end_a,
        fade_len,
        period,
    ).unwrap();

    let windows_b = find_window(buf2, None);
    let marks_b = psola::generate_pitch_marks(buf2, &windows_b);

    let (voiced_start_b, voiced_end_b) = find_voiced_region(&buf2).unwrap();
    let start_b = find_start_b(
        &marks_b,
        voiced_start_b,
        fade_len,
        period,
        buf2.samples.len(),
    ).unwrap();

    assert!(cut_a >= fade_len);
    assert!(cut_a <= voiced_end_a);

    assert!(start_b >= voiced_start_b);
    assert!(start_b + fade_len <= buf2.samples.len());

    phase_aligned_crossfade(buf1, buf2, cut_a, start_b, period, fade_len)
}

pub fn find_cut_a(
    marks: &[usize],
    voiced_end: usize,
    fade_len: usize,
    period: usize,
) -> Option<usize> {
    marks
        .iter()
        .copied()
        .rev()
        .find(|&m| m + period <= voiced_end && m >= fade_len)
}

pub fn find_start_b(
    marks: &[usize],
    voiced_start: usize,
    fade_len: usize,
    period: usize,
    buffer_len: usize,
) -> Option<usize> {
    marks
        .iter()
        .copied()
        .find(|&m| {
            m >= voiced_start &&
            m + fade_len + period <= buffer_len
        })
}

pub fn phase_aligned_crossfade(
    buf1: &AudioBuffer,
    buf2: &AudioBuffer,
    cut_a: usize,
    start_b: usize,
    period: usize,
    fade_len: usize,
) -> AudioBuffer {
    assert_eq!(buf1.sample_rate, buf2.sample_rate);

    let fade_len = fade_len.max(5 * period);

    let start_b = align_b_start(
        &buf1.samples,
        &buf2.samples,
        cut_a,
        start_b,
        period,
    );

    let a_start = cut_a - fade_len;
    let b_start = start_b;

    assert!(a_start + fade_len <= buf1.samples.len());
    assert!(b_start + fade_len <= buf2.samples.len());

    let mut out = Vec::with_capacity(
        a_start + fade_len + (buf2.samples.len() - (b_start + fade_len))
    );

    out.extend_from_slice(&buf1.samples[..a_start]);

    for i in 0..fade_len {
        let t = i as f32 / (fade_len - 1) as f32;
        let fade_out = (FRAC_PI_2 * t).cos();
        let fade_in  = (FRAC_PI_2 * t).sin();

        let va = buf1.samples[a_start + i] * fade_out;
        let vb = buf2.samples[b_start + i] * fade_in;

        out.push(va + vb);
    }

    out.extend_from_slice(&buf2.samples[b_start + fade_len..]);

    AudioBuffer {
        sample_rate: buf1.sample_rate,
        samples: out,
    }
}

pub fn align_b_start(
    a: &[f32],
    b: &[f32],
    cut_a: usize,
    start_b: usize,
    period: usize,
) -> usize {
    let n = period.max(1); // template length

    // Safety
    if cut_a < n || start_b + n >= b.len() {
        return start_b;
    }

    let a_ref = &a[cut_a - n..cut_a];

    let mut best = start_b;
    let mut best_score = f32::NEG_INFINITY;

    let search = period; // ±1 period
    for d in -(search as isize)..=(search as isize) {
        let cand = start_b as isize + d;
        if cand < 0 {
            continue;
        }
        let cand = cand as usize;
        if cand + n > b.len() {
            continue;
        }

        let b_ref = &b[cand..cand + n];
        let score = corr(a_ref, b_ref);

        if score > best_score {
            best_score = score;
            best = cand;
        }
    }

    best
}

pub fn corr(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    let mut s = 0.0;
    for i in 0..a.len() {
        s += a[i] * b[i];
    }
    s
}

pub fn find_voiced_region(buf: &AudioBuffer) -> Option<(usize, usize)> {
    let win = (0.010 * buf.sample_rate as f32) as usize; // 10 ms
    let hop = (0.005 * buf.sample_rate as f32) as usize; // 5 ms
    if buf.len() < win {
        return None;
    }

    let mut rms_vals = Vec::new();
    let mut max_rms = 0.0f32;

    let mut pos = 0;
    while pos + win <= buf.len() {
        let mut sum = 0.0;
        for &s in &buf.samples[pos..pos + win] {
            sum += s * s;
        }
        let rms = (sum / win as f32).sqrt();
        max_rms = max_rms.max(rms);
        rms_vals.push((pos, rms));
        pos += hop;
    }

    let thresh = 0.05 * max_rms;

    let mut start = None;
    let mut end = None;

    for (pos, rms) in rms_vals {
        if rms >= thresh {
            if start.is_none() {
                start = Some(pos);
            }
            end = Some(pos + win);
        }
    }

    match (start, end) {
        (Some(s), Some(e)) => Some((s, e.min(buf.len()))),
        _ => None,
    }
}
