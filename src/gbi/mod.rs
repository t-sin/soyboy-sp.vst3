pub type Signal = (f64, f64);

// 0.0 <= ph < 1.0
// 0.0 <= duty < 1.0
fn pulse(phase: f64, duty: f64) -> f64 {
    let ph = phase % 1.0;
    if ph < duty {
        -1.0
    } else {
        1.0
    }
}

pub struct GameBoyInstrument {
    pub phase: f64,
}

impl GameBoyInstrument {
    pub fn new() -> GameBoyInstrument {
        GameBoyInstrument { phase: 0.0 }
    }

    pub fn process(&mut self, sample_rate: f64) -> Signal {
        let freq = 440.0;
        let phase_diff = (freq / sample_rate) / 2.0;

        let v = pulse(self.phase, 0.5);
        self.phase += phase_diff;

        (v, v)
    }
}
