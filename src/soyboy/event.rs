use crate::common::i4;

pub enum Event {
    NoteOn { note: u16, velocity: f64 },
    NoteOff { note: u16 },
    PitchBend { ratio: f64 },
    SweepReset { freq: f64 },
    SetWaveTable { idx: usize, value: i4 },
    ResetWaveTableAsSine,
    ResetWaveTableAtRandom,
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
            4 => Ok(Event::SetWaveTable {
                idx: 0,
                value: i4::from(0i8),
            }),
            5 => Ok(Event::ResetWaveTableAsSine),
            6 => Ok(Event::ResetWaveTableAtRandom),
            _ => Err(()),
        }
    }
}

pub trait Triggered {
    fn trigger(&mut self, event: &Event);
}
