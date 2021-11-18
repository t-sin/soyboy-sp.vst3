pub enum Event {
    NoteOn { note: u16, velocity: f64 },
    NoteOff { note: u16 },
    PitchBend { ratio: f64 },
}

pub trait Triggered {
    fn trigger(&mut self, event: &Event);
}
