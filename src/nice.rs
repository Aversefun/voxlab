//! Nice stuff to have.

/// Linear interpolation
pub fn lerp(x: f32, y: f32, t: f32) -> f32 {
    ((1.0 - t) * x) + (t * y)
}

pub fn lerp_list(list: impl AsRef<[f32]>, t: f32) -> f32 {
    let list = list.as_ref();
    if t < 0.0 || t > list.len() as f32 {
        panic!("out of range")
    }
    if t.is_nan() {
        panic!("NaN")
    }
    lerp(
        list[t.trunc() as usize],
        list[t.trunc() as usize + 1],
        t.fract(),
    )
}
