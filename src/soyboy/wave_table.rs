use crate::soyboy::{
    event::{Event, Triggered},
    parameters::{Parameter, Parametric},
    types::{i4, AudioProcessor},
    utils::{frequency_from_note_number, level_from_velocity},
};

const WAVETABLE_SIZE: usize = 32;
const WAVETABLE_SIZE_F64: f64 = WAVETABLE_SIZE as f64;

pub struct WaveTableOscillator {
    phase: f64,
    freq: f64,
    pitch: f64,
    velocity: f64,

    table: [i4; WAVETABLE_SIZE],
}

impl WaveTableOscillator {
    pub fn new() -> Self {
        let mut table = [i4::from(0.0); WAVETABLE_SIZE];
        let mut phase: f64 = 0.0;
        for e in table.iter_mut() {
            let v = (phase * 2.0 * std::f64::consts::PI).sin() * i4::max();
            *e = i4::from(v);
            phase += 1.0 / WAVETABLE_SIZE as f64;
        }

        WaveTableOscillator {
            phase: 0.0,
            freq: 0.0,
            pitch: 0.0,
            velocity: 0.0,

            table: table,
        }
    }
}

impl Triggered for WaveTableOscillator {
    fn trigger(&mut self, event: &Event) {
        match event {
            Event::NoteOn { note, velocity } => {
                self.freq = frequency_from_note_number(*note);
                self.velocity = *velocity;
            }
            Event::NoteOff { note: _ } => {}
            Event::PitchBend { ratio } => {
                self.pitch = *ratio;
            }
        }
    }
}

impl Parametric<Parameter> for WaveTableOscillator {
    fn set_param(&mut self, _param: &Parameter, _value: f64) {}
    fn get_param(&self, _param: &Parameter) -> f64 {
        0.0
    }
}

impl AudioProcessor<i4> for WaveTableOscillator {
    fn process(&mut self, sample_rate: f64) -> i4 {
        let v = self.table[self.phase as usize] * level_from_velocity(self.velocity);

        let phase_diff = (self.freq / sample_rate) * WAVETABLE_SIZE_F64;
        self.phase = (self.phase + phase_diff) % WAVETABLE_SIZE_F64;

        i4::from(v)
    }
}
