use std::convert::TryFrom;

use crate::soyboy::{
    envelope_generator::{EnvelopeGenerator, EnvelopeState},
    noise::NoiseOscillator,
    parameters::{Parameter, Parametric},
    square_wave::SquareWaveOscillator,
    types::{AudioProcessor, Oscillator},
    utils::level,
};

pub type Signal = (f64, f64);

#[derive(Copy, Clone)]
enum OscillatorType {
    Square = 0,
    Noise,
    WaveTable,
}

impl TryFrom<u32> for OscillatorType {
    type Error = ();

    fn try_from(id: u32) -> Result<Self, Self::Error> {
        if id == OscillatorType::Square as u32 {
            Ok(OscillatorType::Square)
        } else if id == OscillatorType::Noise as u32 {
            Ok(OscillatorType::Noise)
        } else if id == OscillatorType::WaveTable as u32 {
            Ok(OscillatorType::WaveTable)
        } else {
            Err(())
        }
    }
}

pub struct SoyBoy {
    square_osc: SquareWaveOscillator,
    noise_osc: NoiseOscillator,
    envelope_gen: EnvelopeGenerator,
    master_volume: f64,
    selected_osc: OscillatorType,
}

impl AudioProcessor<Signal> for SoyBoy {
    fn process(&mut self, sample_rate: f64) -> Signal {
        let sq_osc = self.square_osc.process(sample_rate).to_f64();
        let n_osc = self.noise_osc.process(sample_rate).to_f64();
        let osc = match self.selected_osc {
            OscillatorType::Square => sq_osc,
            OscillatorType::Noise => n_osc,
            OscillatorType::WaveTable => 0.0,
        };

        let env = self.envelope_gen.process(sample_rate);

        let signal = osc * env * 0.25 * level(self.master_volume);
        (signal, signal)
    }
}

impl SoyBoy {
    pub fn new() -> SoyBoy {
        SoyBoy {
            square_osc: SquareWaveOscillator::new(),
            noise_osc: NoiseOscillator::new(),
            envelope_gen: EnvelopeGenerator::new(),
            master_volume: 1.0,
            selected_osc: OscillatorType::Square,
        }
    }

    pub fn note_on(&mut self, pitch: i16, velocity: f32) {
        self.square_osc.set_pitch(pitch);
        self.square_osc.set_velocity(velocity);

        self.noise_osc.set_velocity(velocity);

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
            Parameter::OscillatorType => {
                if let Ok(r#type) = OscillatorType::try_from(value as u32) {
                    self.selected_osc = r#type
                }
            }
            Parameter::EgAttack => self.envelope_gen.set_param(param, value),
            Parameter::EgDecay => self.envelope_gen.set_param(param, value),
            Parameter::EgSustain => self.envelope_gen.set_param(param, value),
            Parameter::EgRelease => self.envelope_gen.set_param(param, value),
            Parameter::OscSqDuty => self.square_osc.set_param(param, value),
            Parameter::OscNsInterval => self.noise_osc.set_param(param, value),
        }
    }

    fn get_param(&self, param: &Parameter) -> f64 {
        match param {
            Parameter::MasterVolume => self.master_volume,
            Parameter::OscillatorType => {
                let v = self.selected_osc as u32;
                v.into()
            }
            Parameter::EgAttack => self.envelope_gen.get_param(param),
            Parameter::EgDecay => self.envelope_gen.get_param(param),
            Parameter::EgSustain => self.envelope_gen.get_param(param),
            Parameter::EgRelease => self.envelope_gen.get_param(param),
            Parameter::OscSqDuty => self.square_osc.get_param(param),
            Parameter::OscNsInterval => self.noise_osc.get_param(param),
        }
    }
}
