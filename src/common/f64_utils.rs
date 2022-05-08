pub fn is_normal(v: f64) -> bool {
    v.abs() < 0.001
}

pub fn normalize(v: f64) -> f64 {
    if is_normal(v) {
        0.0
    } else {
        v
    }
}
