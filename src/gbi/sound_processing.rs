use crate::gbi::types::i4;

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
