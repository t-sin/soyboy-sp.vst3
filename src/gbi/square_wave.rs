use crate::gbi::types::{u4, AudioProcessor};

fn pulse(phase: f64, duty: f64) -> u4 {
    let ph = phase % 1.0;
    if ph < duty {
        u4::MIN
    } else {
        u4::MAX
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

impl AudioProcessor<u4> for SquareWaveOscillator {
    fn process(&mut self, sample_rate: f64) -> u4 {
        let freq = 440.0;
        let phase_diff = (freq / sample_rate) / 2.0;

        let v = pulse(self.phase, 0.5);
        self.phase += phase_diff;
        v
    }
}
