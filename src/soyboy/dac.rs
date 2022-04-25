use crate::soyboy::types::i4;

pub struct DAConverter {
    freq: f64,
    q: f64,

    input_buf: [f64; 2],
    output_buf: [f64; 2],
    a0: f64,
    a1: f64,
    a2: f64,
    b0: f64,
    b1: f64,
    b2: f64,
    initialized: bool,
}

impl DAConverter {
    pub fn new(freq: f64, q: f64) -> Self {
        DAConverter {
            freq,
            q,

            input_buf: [0.0; 2],
            output_buf: [0.0; 2],
            a0: 0.0,
            a1: 0.0,
            a2: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            initialized: false,
        }
    }

    fn calculate_coefficient(&mut self, sample_rate: f64) {
        let w = (2.0 * std::f64::consts::PI * self.freq) / sample_rate;

        let (sw, cw) = (w.sin(), w.cos());
        let a = sw / (2.0 * self.q);

        self.b0 = (1.0 - cw) / 2.0;
        self.b1 = 1.0 - cw;
        self.b2 = (1.0 - cw) / 2.0;

        self.a0 = 1.0 + a;
        self.a1 = -2.0 * cw;
        self.a2 = 1.0 - a;
    }

    pub fn process(&mut self, sample_rate: f64, input: i4) -> f64 {
        if !self.initialized {
            self.calculate_coefficient(sample_rate);
            self.initialized = true;
        }

        let input: f64 = input.into();
        let input = input / i4::max();
        let (in0, in1) = (self.input_buf[0], self.input_buf[1]);
        let (out0, out1) = (self.output_buf[0], self.output_buf[1]);

        let output =
            (self.b0 / self.a0 * input) + (self.b1 / self.a0 * in0) + (self.b2 / self.a0 * in1)
                - (self.a1 / self.a0 * out0)
                - (self.a2 / self.a0 * out1);

        self.input_buf[1] = self.input_buf[0];
        self.input_buf[0] = input;
        self.output_buf[1] = self.output_buf[0];
        self.output_buf[0] = output;

        output
    }
}
