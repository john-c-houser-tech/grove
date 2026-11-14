//! Small curves shared by the sky, the weather and the sounds.

/// `v` held between 0 and 1.
pub fn clamp01(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}

/// 0 up to `a`, 1 from `b`, and an S-shaped ease between.
pub fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let t = clamp01((x - a) / (b - a));
    t * t * (3.0 - 2.0 * t)
}
