use std::convert::TryFrom;

use crate::soyboy::{
    envelope_generator::{EnvelopeGenerator, EnvelopeState},
    parameters::{Parameter, Parametric},
    square_wave::{SquareWaveDuty, SquareWaveOscillator},
    types::{AudioProcessor, Oscillator},
    utils::level,
};

pub type Signal = (f64, f64);

pub struct SoyBoy {
    square_osc: SquareWaveOscillator,
    envelope_gen: EnvelopeGenerator,
    master_volume: f64,
}

impl AudioProcessor<Signal> for SoyBoy {
    fn process(&mut self, sample_rate: f64) -> Signal {
        let osc = self.square_osc.process(sample_rate).to_f64();
        let env = self.envelope_gen.process(sample_rate);

        let signal = osc * env * 0.4 * level(self.master_volume);
        (signal, signal)
    }
}

impl SoyBoy {
    pub fn new() -> SoyBoy {
        SoyBoy {
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

impl Parametric<Parameter> for SoyBoy {
    fn set_param(&mut self, param: &Parameter, value: f64) {
        match param {
            Parameter::MasterVolume => self.master_volume = value,
            Parameter::EgAttack => self.envelope_gen.attack_time = value,
            Parameter::EgDecay => self.envelope_gen.decay_time = value,
            Parameter::EgSustain => self.envelope_gen.sustain_val = value,
            Parameter::EgRelease => self.envelope_gen.release_time = value,
            Parameter::OscSqDuty => {
                if let Ok(ratio) = SquareWaveDuty::try_from(value as u32) {
                    self.square_osc.set_duty(ratio);
                } else {
                }
            }
        }
    }
}
