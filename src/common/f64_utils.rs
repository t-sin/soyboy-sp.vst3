pub fn normalize(v: f64) -> f64 {
    if v.is_subnormal() {
        0.0
    } else {
        v
    }
}
