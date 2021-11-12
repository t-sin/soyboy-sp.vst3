use crate::soyboy::{
    envelope_generator::{EnvelopeGenerator, EnvelopeState},
    parameters::{Parameter, Parametric},
    square_wave::SquareWaveOscillator,
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

    pub fn note_on(&mut self, pitch: i16, velocity: f32) {
        self.square_osc.set_pitch(pitch);
        self.square_osc.set_velocity(velocity);
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
            Parameter::EgAttack => self.envelope_gen.set_param(param, value),
            Parameter::EgDecay => self.envelope_gen.set_param(param, value),
            Parameter::EgSustain => self.envelope_gen.set_param(param, value),
            Parameter::EgRelease => self.envelope_gen.set_param(param, value),
            Parameter::OscSqDuty => self.square_osc.set_param(param, value),
        }
    }
}
