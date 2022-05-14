use crate::common::{f64_utils, i4};

pub fn linear(x: f64, slope: f64) -> f64 {
    let v = x * slope;
    f64_utils::normalize(v)
}

/// This maps from continuous value `x` to discrete value.
/// This is for getting rough 4bit envelope signals.
pub fn discrete_loudness(x: f64) -> f64 {
    let v = ((x * 16.0) as u32) as f64 / 16.0;
    f64_utils::normalize(v)
}

pub fn pulse(phase: f64, duty: f64) -> i4 {
    let ph = phase % 1.0;

    if ph < duty {
        i4::from(i4::SIGNED_MIN)
    } else {
        i4::from(i4::SIGNED_MAX)
    }
}

const NOTE_NUMBER_OF_440_HZ: i16 = 69;

/// https://steinbergmedia.github.io/vst3_doc/vstinterfaces/structSteinberg_1_1Vst_1_1NoteOnEvent.html の pitch の項目
pub fn frequency_from_note_number(note_num: u16) -> f64 {
    440.0 * 2.0_f64.powf((note_num as i16 - NOTE_NUMBER_OF_440_HZ) as f64 / 12.0)
}

pub fn ratio_from_cents(cents: i16) -> f64 {
    2.0f64.powf(cents as f64 / 1200.0)
}

pub fn level(decibel: f64) -> f64 {
    let v = 10.0f64.powf(decibel / 10.0);
    f64_utils::normalize(v)
}

pub fn level_from_velocity(velocity: f64) -> f64 {
    let v = 10.0f64.powf(velocity - 1.0);
    f64_utils::normalize(v)
}

pub fn linear_denormalize(v: f64, min: f64, max: f64) -> f64 {
    let range = (max - min).abs();
    let v = v * range + min;
    f64_utils::normalize(v)
}

pub fn linear_normalize(x: f64, min: f64, max: f64) -> f64 {
    let range = (max - min).abs();
    let v = (x - min) / range;
    f64_utils::normalize(v)
}

pub fn divergent_denormalize(v: f64, min: f64, max: f64, factor: f64) -> f64 {
    let range = max.abs() + min.abs();
    let v = range * v.powf(factor) + min;
    f64_utils::normalize(v)
}

pub fn divergent_normalize(x: f64, min: f64, max: f64, factor: f64) -> f64 {
    let range = max.abs() + min.abs();
    let v = ((x - min) / range).powf(1.0 / factor);
    f64_utils::normalize(v)
}

pub fn convergent_denormalize(v: f64, min: f64, max: f64, factor: f64) -> f64 {
    let range = max.abs() + min.abs();
    let v = range * v.powf(1.0 / factor) + min;
    f64_utils::normalize(v)
}

pub fn convergent_normalize(x: f64, min: f64, max: f64, factor: f64) -> f64 {
    let range = max.abs() + min.abs();
    let v = ((x - min) / range).powf(factor);
    f64_utils::normalize(v)
}
