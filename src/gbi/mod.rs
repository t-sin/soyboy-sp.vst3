mod envelope_generator;
mod sound_processing;
mod square_wave;
mod types;

use envelope_generator::{EnvelopeGenerator, EnvelopeState};
use square_wave::SquareWaveOscillator;
pub use types::{AudioProcessor, Oscillator};

pub type Signal = (f64, f64);

pub struct GameBoyInstrument {
    square_osc: SquareWaveOscillator,
    envelope_gen: EnvelopeGenerator,
}

impl AudioProcessor<Signal> for GameBoyInstrument {
    fn process(&mut self, sample_rate: f64) -> Signal {
        let osc = self.square_osc.process(sample_rate).to_f64();
        let env = self.envelope_gen.process(sample_rate);

        let signal = osc * env * 0.5;
        (signal, signal)
    }
}

impl GameBoyInstrument {
    pub fn new() -> GameBoyInstrument {
        GameBoyInstrument {
            square_osc: SquareWaveOscillator::new(),
            envelope_gen: EnvelopeGenerator::new(),
        }
    }

    pub fn note_on(&mut self, pitch: i16) {
        self.square_osc.set_pitch(pitch);
        self.envelope_gen.state = EnvelopeState::On;
    }

    pub fn note_off(&mut self) {
        self.envelope_gen.state = EnvelopeState::Off;
    }
}
