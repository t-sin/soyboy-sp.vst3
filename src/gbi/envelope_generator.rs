use crate::gbi::types::AudioProcessor;

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

    fn update_state(&mut self, sample_rate: f64) {
        let s = self.elapsed_samples as f64;

        match self.state {
            EnvelopeState::Attack => {
                let attack_samples = self.attack_time * sample_rate;
                if s > attack_samples {
                    self.set_state(EnvelopeState::Decay);
                }
            }
            EnvelopeState::Decay => {
                let decay_samples = self.decay_time * sample_rate;
                if s > decay_samples {
                    self.set_state(EnvelopeState::Sustain);
                }
            }
            EnvelopeState::Sustain => (),
            EnvelopeState::Release => {
                let release_samples = self.release_time * sample_rate;
                if s > release_samples {
                    self.set_state(EnvelopeState::Off);
                }
            }
            EnvelopeState::Off => (),
        };
    }

    fn calculate(&mut self, sample_rate: f64) -> f64 {
        let s = self.elapsed_samples as f64;

        match self.state {
            EnvelopeState::Attack => {
                let attack_samples = self.attack_time * sample_rate;
                let v = s / attack_samples;

                v
            }
            EnvelopeState::Decay => {
                let decay_samples = self.decay_time * sample_rate;
                let max = self.last_state_value - self.sustain_val;
                let v = self.last_state_value - max * (s / decay_samples);

                v
            }
            EnvelopeState::Sustain => self.sustain_val,
            EnvelopeState::Release => {
                let release_samples = self.release_time * sample_rate;
                let max = self.last_state_value;
                let v = max - max * (s / release_samples);

                v
            }
            EnvelopeState::Off => 0.0,
        }
    }
}

impl AudioProcessor<f64> for EnvelopeGenerator {
    fn process(&mut self, sample_rate: f64) -> f64 {
        self.update_state(sample_rate);
        let v = self.calculate(sample_rate);
        self.last_value = v;
        self.elapsed_samples += 1;

        v
    }
}
