mod types;

use types::u4;

fn pulse(phase: f64, duty: f64) -> u4 {
    let ph = phase % 1.0;
    if ph < duty {
        u4::MIN
    } else {
        u4::MAX
    }
}

pub trait AudioProcessor<T> {
    fn process(&mut self, sample_rate: f64) -> T;
}

pub struct SquareWaveOscillator {
    pub phase: f64,
}

impl SquareWaveOscillator {
    pub fn new() -> Self {
        SquareWaveOscillator { phase: 0.0 }
    }
}

impl AudioProcessor<u4> for SquareWaveOscillator {
    fn process(&mut self, sample_rate: f64) -> u4 {
        let freq = 440.0;
        let phase_diff = (freq / sample_rate) / 2.0;

        let v = pulse(self.phase, 0.5);
        self.phase += phase_diff;
        v
    }
}

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
