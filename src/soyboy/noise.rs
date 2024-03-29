use rand::prelude::*;

use crate::{
    common::i4,
    soyboy::{
        event::{Event, Triggered},
        parameters::{ParameterDef, Parametric, SoyBoyParameter},
        types::AudioProcessor,
    },
};

const TABLE_SIZE: usize = 1024 * 8;

pub struct NoiseOscillator {
    interval_msec: f64,
    sec_counter: f64,
    table: [i4; TABLE_SIZE],
    table_index: usize,
}

impl NoiseOscillator {
    pub fn new() -> Self {
        let mut table = [i4::from(0.0); TABLE_SIZE];
        for e in table.iter_mut() {
            let v = random::<f64>() * 2.0 - 1.0;
            *e = i4::from(v);
        }

        NoiseOscillator {
            interval_msec: 0.1,
            sec_counter: 0.0,
            table,
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

        self.table[self.table_index]
    }

    fn set_freq(&mut self, _freq: f64) {}
}

impl Triggered for NoiseOscillator {
    fn trigger(&mut self, event: &Event) {
        if let Event::NoteOn {
            note: _,
            velocity: _,
        } = event
        {}
    }
}

impl Parametric<SoyBoyParameter> for NoiseOscillator {
    fn set_param(&mut self, param: &SoyBoyParameter, _param_def: &ParameterDef, value: f64) {
        if param == &SoyBoyParameter::OscNsInterval {
            self.interval_msec = value;
        }
    }

    fn get_param(&self, param: &SoyBoyParameter) -> f64 {
        match param {
            SoyBoyParameter::OscNsInterval => self.interval_msec,
            _ => 0.0,
        }
    }
}
