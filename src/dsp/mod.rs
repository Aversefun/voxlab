//! Audio abuse.

pub mod crossfade;
pub mod pitch;
pub mod psola;
pub mod stretch;
pub mod window;
pub mod window_calc;

pub fn low_pass(values: Vec<f32>, alpha: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; values.len()];
    out[0] = values[0];

    for i in 1..values.len() {
        out[i] = out[i - 1] + alpha * (values[i] - out[i - 1]);
    }

    out
}
