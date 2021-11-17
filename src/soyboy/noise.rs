use rand::prelude::*;

use crate::soyboy::{
    parameters::{Parameter, Parametric},
    types::{i4, AudioProcessor, Oscillator},
    utils,
};

const TABLE_SIZE: usize = 1024 * 8;

pub struct NoiseOscillator {
    velocity: f64,
    interval_msec: f64,
    sec_counter: f64,
    table: [i8; TABLE_SIZE],
    table_index: usize,
}

impl NoiseOscillator {
    pub fn new() -> Self {
        let mut table = [0; TABLE_SIZE];
        for idx in 0..table.len() {
            let range = (i4::MAX_I8 - i4::MIN_I8) as f64;
            let v = (random::<f64>() * range + i4::MIN_I8 as f64) as i8;
            table[idx] = v;
        }

        NoiseOscillator {
            velocity: 0.0,
            interval_msec: 0.1,
            sec_counter: 0.0,
            table: table,
            table_index: 0,
        }
    }
}

impl AudioProcessor<i4> for NoiseOscillator {
    fn process(&mut self, sample_rate: f64) -> i4 {
        if self.sec_counter >= self.interval_msec / 1000.0 {
            self.table_index = (self.table_index + 1) % self.table.len();
            self.sec_counter = 0.0;
        }

        self.sec_counter += 1.0 / sample_rate;

        let v =
            (self.table[self.table_index] as f64 * utils::level_from_velocity(self.velocity)) as i8;
        i4::from(v)
    }
}

impl Parametric<Parameter> for NoiseOscillator {
    fn set_param(&mut self, param: &Parameter, value: f64) {
        match param {
            Parameter::OscNsInterval => self.interval_msec = value,
            _ => (),
        }
    }

    fn get_param(&self, param: &Parameter) -> f64 {
        match param {
            Parameter::OscNsInterval => self.interval_msec,
            _ => 0.0,
        }
    }
}

impl Oscillator for NoiseOscillator {
    fn set_pitch(&mut self, _note: u16) {}

    fn set_velocity(&mut self, velocity: f64) {
        self.velocity = velocity;
    }
}
