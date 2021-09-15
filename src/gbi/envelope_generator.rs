use crate::gbi::types::AudioProcessor;

pub enum EnvelopeState {
    Off,
    On,
}

pub struct EnvelopeGenerator {
    pub state: EnvelopeState,
}

impl EnvelopeGenerator {
    pub fn new() -> EnvelopeGenerator {
        EnvelopeGenerator {
            state: EnvelopeState::Off,
        }
    }
}

impl AudioProcessor<f64> for EnvelopeGenerator {
    fn process(&mut self, _sample_rate: f64) -> f64 {
        match self.state {
            EnvelopeState::Off => 0.0,
            EnvelopeState::On => 1.0,
        }
    }
}
