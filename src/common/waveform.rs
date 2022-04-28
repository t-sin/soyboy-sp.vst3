use super::constants::OSCILLOSCOPE_SAIMPLE_SIZE;

#[derive(Clone)]
pub struct Waveform {
    signals: [f64; OSCILLOSCOPE_SAIMPLE_SIZE],
    idx: usize,
    skips: usize,
}

impl Waveform {
    pub fn new() -> Self {
        Self {
            signals: [0.0; OSCILLOSCOPE_SAIMPLE_SIZE],
            idx: 0,
            skips: 0,
        }
    }

    pub fn set_signal(&mut self, v: f64) {
        if self.skips == 32 {
            self.signals[self.idx] = v;
            self.idx = (self.idx + 1) % self.signals.len();
            self.skips = 0;
        } else {
            self.skips += 1;
        }
    }

    pub fn set_signals(&mut self, signals: &[f64]) {
        (&mut self.signals).copy_from_slice(&signals[..OSCILLOSCOPE_SAIMPLE_SIZE]);
    }

    pub fn get_signals(&self) -> &[f64] {
        &self.signals
    }
}

impl PartialEq for Waveform {
    fn eq(&self, _other: &Self) -> bool {
        false
    }

    fn ne(&self, _other: &Self) -> bool {
        true
    }
}
impl Eq for Waveform {}
