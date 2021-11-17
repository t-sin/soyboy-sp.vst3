pub enum Event {
    NoteOn { note: u16, velocity: f64 },
    NoteOff { note: u16 },
}

pub trait Triggered {
    fn trigger(&mut self, event: &Event);
}
