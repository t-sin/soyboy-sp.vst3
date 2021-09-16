pub fn frequency_from_note_number(note_num: u16, note_num_440hz: u16) -> f64 {
    440.0 * 2.0_f64.powf((note_num - note_num_440hz) as f64 / 12.0)
}
