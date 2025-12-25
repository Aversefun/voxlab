//! Nice stuff to have.

/// Linear interpolation
pub fn lerp(x: f32, y: f32, t: f32) -> f32 {
    ((1.0 - t) * x) + (t * y)
}
