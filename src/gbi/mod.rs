mod square_wave;
mod types;

use square_wave::SquareWaveOscillator;
pub use types::AudioProcessor;

pub type Signal = (f64, f64);

pub struct GameBoyInstrument {
    square_osc: SquareWaveOscillator,
}

impl AudioProcessor<Signal> for GameBoyInstrument {
    fn process(&mut self, sample_rate: f64) -> Signal {
        let v = self.square_osc.process(sample_rate).to_f64();
        (v, v)
    }
}

impl GameBoyInstrument {
    pub fn new() -> GameBoyInstrument {
        GameBoyInstrument {
            square_osc: SquareWaveOscillator::new(),
        }
    }
}
