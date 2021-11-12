use crate::soyboy::types::i4;

pub fn linear(x: f64, slope: f64) -> f64 {
    x * slope
}

/// This maps from continuous value `x` to discrete value.
/// This is for getting rough 4bit envelope signals.
pub fn discrete_loudness(x: f64) -> f64 {
    let v = ((x * 16.0) as u32) as f64 / 16.0;
    v
}

pub fn pulse(phase: f64, duty: f64) -> i4 {
    let ph = phase % 1.0;
    if ph < duty {
        i4::MIN
    } else {
        i4::MAX
    }
}

pub fn frequency_from_note_number(note_num: u16, note_num_440hz: u16) -> f64 {
    440.0 * 2.0_f64.powf((note_num as i32 - note_num_440hz as i32) as f64 / 12.0)
}

pub fn level(decibel: f64) -> f64 {
    let level = 10.0f64.powf(decibel / 10.0);
    level
}

pub fn linear_denormalize(v: f64, min: f64, max: f64) -> f64 {
    let range = max.abs() - min.abs();
    v * range + min
}

pub fn linear_normalize(x: f64, min: f64, max: f64) -> f64 {
    let range = max.abs() - min.abs();
    (x - min) / range
}

pub fn divergent_denormalize(v: f64, min: f64, max: f64, factor: f64) -> f64 {
    let range = max.abs() + min.abs();
    let x = range * v.powf(factor) + min;
    x
}

pub fn divergent_normalize(x: f64, min: f64, max: f64, factor: f64) -> f64 {
    let range = max.abs() + min.abs();
    let v = ((x - min) / range).powf(1.0 / factor);
    v
}

pub fn convergent_denormalize(v: f64, min: f64, max: f64, factor: f64) -> f64 {
    let range = max.abs() + min.abs();
    let x = range * v.powf(1.0 / factor) + min;
    x
}

pub fn convergent_normalize(x: f64, min: f64, max: f64, factor: f64) -> f64 {
    let range = max.abs() + min.abs();
    let v = ((x - min) / range).powf(factor);
    v
}
