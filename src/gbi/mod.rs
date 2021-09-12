pub type Signal = (f64, f64);

pub struct GameBoyInstrument {
    pub ph: f64,
}

impl GameBoyInstrument {
    pub fn new() -> GameBoyInstrument {
        GameBoyInstrument { ph: 0.0 }
    }

    pub fn process(&mut self) -> Signal {
        let v = self.ph.sin();
        self.ph += 0.05;

        (v, v)
    }
}
