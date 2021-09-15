use crate::gbi::types::{i4, AudioProcessor};

fn pulse(phase: f64, duty: f64) -> i4 {
    let ph = phase % 1.0;
    if ph < duty {
        i4::MIN
    } else {
        i4::MAX
    }
}

pub struct SquareWaveOscillator {
    pub phase: f64,
}

impl SquareWaveOscillator {
    pub fn new() -> Self {
        SquareWaveOscillator { phase: 0.0 }
    }
}

impl AudioProcessor<i4> for SquareWaveOscillator {
    fn process(&mut self, sample_rate: f64) -> i4 {
        let freq = 440.0;
        let phase_diff = (freq / sample_rate) / 2.0;

        let v = pulse(self.phase, 0.5);
        self.phase += phase_diff;
        v
    }
}
