use crate::soyboy::parameters::{Parameter, Parametric};
use crate::soyboy::types::AudioProcessor;
use crate::soyboy::utils::{discrete_loudness, linear};

#[derive(Debug)]
pub enum EnvelopeState {
    Attack,
    Decay,
    Sustain,
    Release,
    Off,
}

pub struct EnvelopeGenerator {
    pub attack_time: f64,
    pub decay_time: f64,
    pub sustain_val: f64,
    pub release_time: f64,

    state: EnvelopeState,
    elapsed_samples: u64,
    last_value: f64,
    last_state_value: f64,
}

impl EnvelopeGenerator {
    pub fn new() -> EnvelopeGenerator {
        EnvelopeGenerator {
            attack_time: 0.05,
            decay_time: 0.05,
            sustain_val: 0.3,
            release_time: 0.1,

            state: EnvelopeState::Off,
            elapsed_samples: 1,
            last_value: 0.0,
            last_state_value: 0.0,
        }
    }

    pub fn set_state(&mut self, state: EnvelopeState) {
        match self.state {
            EnvelopeState::Attack => self.last_state_value = self.last_value,
            EnvelopeState::Decay => self.last_state_value = self.last_value,
            EnvelopeState::Sustain => self.last_state_value = self.last_value,
            _ => (),
        }
        self.state = state;
        self.elapsed_samples = 0;
    }

    fn update_state(&mut self, s: f64) {
        match self.state {
            EnvelopeState::Attack => {
                if s > self.attack_time {
                    self.set_state(EnvelopeState::Decay);
                    self.last_state_value = 1.0;
                }
            }
            EnvelopeState::Decay => {
                if s > self.decay_time {
                    self.set_state(EnvelopeState::Sustain);
                }
            }
            EnvelopeState::Sustain => (),
            EnvelopeState::Release => {
                if s > self.release_time {
                    self.set_state(EnvelopeState::Off);
                }
            }
            EnvelopeState::Off => (),
        };
    }

    fn calculate(&mut self, s: f64) -> f64 {
        match self.state {
            EnvelopeState::Attack => {
                let v = linear(s, 1.0 / self.attack_time);

                v
            }
            EnvelopeState::Decay => {
                let max = self.last_state_value - self.sustain_val;
                let v = self.last_state_value - max * linear(s, 1.0 / self.decay_time);

                v
            }
            EnvelopeState::Sustain => self.sustain_val,
            EnvelopeState::Release => {
                let max = self.last_state_value;
                let v = max - max * linear(s, 1.0 / self.release_time);

                v
            }
            EnvelopeState::Off => 0.0,
        }
    }
}

impl AudioProcessor<f64> for EnvelopeGenerator {
    fn process(&mut self, sample_rate: f64) -> f64 {
        let s = self.elapsed_samples as f64 / sample_rate;

        self.update_state(s);
        let v = self.calculate(s);
        self.last_value = v;
        self.elapsed_samples += 1;

        discrete_loudness(v)
    }
}

impl Parametric<Parameter> for EnvelopeGenerator {
    fn set_param(&mut self, param: &Parameter, value: f64) {
        match param {
            Parameter::EgAttack => self.attack_time = value,
            Parameter::EgDecay => self.decay_time = value,
            Parameter::EgSustain => self.sustain_val = value,
            Parameter::EgRelease => self.release_time = value,
            _ => (),
        }
    }
}
