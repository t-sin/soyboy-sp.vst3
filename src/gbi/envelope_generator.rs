use crate::gbi::types::AudioProcessor;

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
    prev_val: f64,
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
            prev_val: 0.0,
        }
    }

    pub fn set_state(&mut self, state: EnvelopeState) {
        self.state = state;
        self.elapsed_samples = 0;
    }

    fn update_state(&mut self, sample_rate: f64) {
        let s = self.elapsed_samples as f64;

        match self.state {
            EnvelopeState::Attack => {
                let attack_samples = self.attack_time * sample_rate;
                if s > attack_samples {
                    self.state = EnvelopeState::Decay;
                    self.elapsed_samples = 0;
                }
            }
            EnvelopeState::Decay => {
                let decay_samples = self.decay_time * sample_rate;
                if s > decay_samples {
                    self.state = EnvelopeState::Sustain;
                    self.elapsed_samples = 0;
                }
            }
            EnvelopeState::Sustain => (),
            EnvelopeState::Release => {
                let release_samples = self.release_time * sample_rate;
                if s > release_samples {
                    self.state = EnvelopeState::Off;
                    self.elapsed_samples = 0;
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
                self.prev_val = v;

                v
            }
            EnvelopeState::Decay => {
                let decay_samples = self.decay_time * sample_rate;
                let a = 1.0 - self.sustain_val;
                let v = 1.0 - a * (s / decay_samples);
                self.prev_val = v;

                v
            }
            EnvelopeState::Sustain => {
                self.prev_val = self.sustain_val;
                self.sustain_val
            }
            EnvelopeState::Release => {
                let release_samples = self.release_time * sample_rate;
                let a = self.prev_val;
                let v = a - a * (s / release_samples);

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
        self.elapsed_samples += 1;

        v
    }
}
