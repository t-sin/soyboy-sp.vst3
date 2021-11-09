mod envelope_generator;
mod sound_processing;
mod square_wave;
mod types;

use std::convert::TryFrom;

use envelope_generator::{EnvelopeGenerator, EnvelopeState};
use square_wave::SquareWaveOscillator;
pub use types::{AudioProcessor, Oscillator};

pub type Signal = (f64, f64);

pub struct GameBoyInstrument {
    square_osc: SquareWaveOscillator,
    envelope_gen: EnvelopeGenerator,
    master_volume: f64,
}

#[derive(Copy, Clone)]
pub enum Parameter {
    MasterVolume = 0,
}

impl TryFrom<u32> for Parameter {
    type Error = ();

    fn try_from(id: u32) -> Result<Self, Self::Error> {
        if id == Parameter::MasterVolume as u32 {
            Ok(Parameter::MasterVolume)
        } else {
            Err(())
        }
    }
}

pub trait Parametric<Parameter> {
    fn set_param(&mut self, param: &Parameter, value: f64);
    fn get_param(&self, param: &Parameter) -> f64;
}

impl AudioProcessor<Signal> for GameBoyInstrument {
    fn process(&mut self, sample_rate: f64) -> Signal {
        let osc = self.square_osc.process(sample_rate).to_f64();
        let env = self.envelope_gen.process(sample_rate);

        let signal = osc * env * self.master_volume;
        (signal, signal)
    }
}

impl GameBoyInstrument {
    pub fn new() -> GameBoyInstrument {
        GameBoyInstrument {
            square_osc: SquareWaveOscillator::new(),
            envelope_gen: EnvelopeGenerator::new(),
            master_volume: 1.0,
        }
    }

    pub fn note_on(&mut self, pitch: i16) {
        self.square_osc.set_pitch(pitch);
        self.envelope_gen.set_state(EnvelopeState::Attack);
    }

    pub fn note_off(&mut self) {
        self.envelope_gen.set_state(EnvelopeState::Release);
    }
}

impl Parametric<Parameter> for GameBoyInstrument {
    fn set_param(&mut self, param: &Parameter, value: f64) {
        match param {
            Parameter::MasterVolume => self.master_volume = value,
        }
    }

    fn get_param(&self, param: &Parameter) -> f64 {
        match param {
            Parameter::MasterVolume => self.master_volume,
        }
    }
}
