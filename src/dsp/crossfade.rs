//! Crossfade between two buffers.

use std::f32::consts::FRAC_PI_2;

use crate::{
    audio::buffer::AudioBuffer,
    dsp::{
        psola::{self, Grain, get_avg_period, overlap_add_grain},
        window_calc::find_window,
    },
};

pub const CROSSFADE_TIME: usize = 2205;

const MIN_TRANSITION_PERIODS: usize = 5;

/// Grain-level interpolation.
#[derive(Clone, Debug)]
pub struct GrainInterp {
    /// Grain index within the target phoneme
    pub target_grain: usize,
    pub fade_len: usize,
}

pub fn crossfade(buf1: &AudioBuffer, buf2: &AudioBuffer, interp: &GrainInterp) -> AudioBuffer {
    let period = get_avg_period(buf1);

    let fade_len = interp.fade_len.max(5 * period);

    let windows_a = find_window(buf1, None);
    let marks_a = psola::generate_pitch_marks(buf1, &windows_a);

    let (voiced_start_a, voiced_end_a) = find_voiced_region(&buf1).unwrap();
    let cut_a = find_cut_a(&marks_a, voiced_end_a, interp.fade_len, period).unwrap();

    let windows_b = find_window(buf2, None);
    let marks_b = psola::generate_pitch_marks(buf2, &windows_b);

    let (voiced_start_b, voiced_end_b) = find_voiced_region(&buf2).unwrap();
    let start_b = find_start_b(
        &marks_b,
        voiced_start_b,
        interp.fade_len,
        period,
        buf2.samples.len(),
    )
    .unwrap();

    assert!(cut_a >= interp.fade_len);
    assert!(cut_a <= voiced_end_a);

    assert!(start_b >= voiced_start_b);
    assert!(start_b + interp.fade_len <= buf2.samples.len());

    phase_aligned_crossfade(
        buf1,
        buf2,
        cut_a,
        start_b,
        period,
        GrainInterp {
            target_grain: interp.target_grain,
            fade_len,
        },
    )
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
        .find(|&m| m >= voiced_start && m + fade_len + period <= buffer_len)
}

pub fn phase_aligned_crossfade(
    buf1: &AudioBuffer,
    buf2: &AudioBuffer,
    cut_a: usize,
    b_start: usize,
    period: usize,
    interp: GrainInterp,
) -> AudioBuffer {
    assert_eq!(buf1.sample_rate, buf2.sample_rate);

    let fade_len = interp.fade_len;

    let b_start = align_b_start(&buf1.samples, &buf2.samples, cut_a, b_start, period);

    let a_start = cut_a - fade_len;

    let mut out =
        Vec::with_capacity(a_start + fade_len + (buf2.samples.len() - (b_start + fade_len)));

    let windows_a = find_window(buf1, None);
    let marks_a = psola::generate_pitch_marks(buf1, &windows_a);
    let grains_a = psola::extract_grains(&buf1, &marks_a, &windows_a);

    let windows_b = find_window(buf2, None);
    let marks_b = psola::generate_pitch_marks(buf2, &windows_b);
    let grains_b = psola::extract_grains(&buf2, &marks_b, &windows_b);

    let fade_len = fade_len / dbg!(period + 1);
    let a_start = a_start / (period + 1);
    let b_start = b_start / (period + 1);

    assert!(dbg!(a_start) + dbg!(fade_len) <= dbg!(grains_a.len()));
    assert!(b_start + fade_len <= grains_b.len());

    for i in 0..a_start {
        out.push(grains_a[i].samples.to_vec());
    }

    for k in 0..fade_len {
        let t = transition_t(k, dbg!(fade_len));
        let ga = &grains_a[dbg!(dbg!(a_start) + dbg!(k))];
        let gb = &grains_b[k];

        out.push(psola::lerp_grain(ga, gb, t));
    }

    for i in fade_len..grains_b.len() {
        out.push(grains_b[i].samples.to_vec());
    }

    AudioBuffer {
        sample_rate: buf1.sample_rate,
        samples: overlap_add_grain(
            out,
            &[marks_a, marks_b].concat(),
            a_start + fade_len + (buf2.samples.len() - (b_start + fade_len)),
        ),
    }
}

fn transition_t(k: usize, len: usize) -> f32 {
    let mid = len / 2;

    if k == mid || k == mid - 1 {
        0.5
    } else {
        let raw = k as f32 / (len - 1) as f32;
        raw * raw * (3.0 - 2.0 * raw)
    }
}

pub fn align_b_start(a: &[f32], b: &[f32], cut_a: usize, start_b: usize, period: usize) -> usize {
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
