pub enum Event {
    NoteOn { note: u16, velocity: f64 },
    NoteOff { note: u16 },
    PitchBend { ratio: f64 },
    SweepReset { freq: f64 },
    SetWaveTable { idx: usize, value: f64 },
    ResetWaveTableAsSine,
    ResetWaveTableAtRandom,
}

impl Into<u32> for Event {
    fn into(self) -> u32 {
        match self {
            Event::NoteOn { .. } => 0,
            Event::NoteOff { .. } => 1,
            Event::PitchBend { .. } => 2,
            Event::SweepReset { .. } => 3,
            Event::SetWaveTable { .. } => 4,
            Event::ResetWaveTableAsSine => 5,
            Event::ResetWaveTableAtRandom => 6,
        }
    }
}

impl TryFrom<u32> for Event {
    type Error = ();

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Event::NoteOn {
                note: 0,
                velocity: 0.0,
            }),
            1 => Ok(Event::NoteOff { note: 0 }),
            2 => Ok(Event::PitchBend { ratio: 0.0 }),
            3 => Ok(Event::SweepReset { freq: 0.0 }),
            4 => Ok(Event::SetWaveTable { idx: 0, value: 0.0 }),
            5 => Ok(Event::ResetWaveTableAsSine),
            6 => Ok(Event::ResetWaveTableAtRandom),
            _ => Err(()),
        }
    }
}

pub trait Triggered {
    fn trigger(&mut self, event: &Event);
}
