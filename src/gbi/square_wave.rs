use crate::gbi::types::{i4, AudioProcessor, Oscillator};

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
    pub freq: f64,
}

impl SquareWaveOscillator {
    pub fn new() -> Self {
        SquareWaveOscillator {
            phase: 0.0,
            freq: 440.0,
        }
    }
}

impl AudioProcessor<i4> for SquareWaveOscillator {
    fn process(&mut self, sample_rate: f64) -> i4 {
        let phase_diff = (self.freq / sample_rate) / 2.0;

        let v = pulse(self.phase, 0.5);
        self.phase += phase_diff;
        v
    }
}

impl Oscillator for SquareWaveOscillator {
    /// https://steinbergmedia.github.io/vst3_doc/vstinterfaces/structSteinberg_1_1Vst_1_1NoteOnEvent.html の pitch の項目
    fn set_pitch(&mut self, note: i16) {
        self.freq = 440.0 * 2.0_f64.powf((note - 69) as f64 / 12.0);
    }
}
