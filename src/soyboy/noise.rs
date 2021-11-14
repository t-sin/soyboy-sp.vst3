use rand::prelude::*;

use crate::soyboy::{
    parameters::{Parameter, Parametric},
    types::{i4, AudioProcessor, Oscillator},
};

pub struct NoiseOscillator {
    velocity: f32,
    interval_msec: f64,
    sec_counter: f64,
    last_signal: i4,
}

impl NoiseOscillator {
    pub fn new() -> Self {
        NoiseOscillator {
            velocity: 0.0,
            interval_msec: 0.1,
            sec_counter: 0.0,
            last_signal: i4::new(0),
        }
    }
}

impl AudioProcessor<i4> for NoiseOscillator {
    fn process(&mut self, sample_rate: f64) -> i4 {
        if self.sec_counter >= self.interval_msec / 1000.0 {
            let range = (i4::MAX_I8 - i4::MIN_I8) as f32;
            let v = ((random::<f32>() * range + i4::MIN_I8 as f32) * self.velocity) as i8;
            self.last_signal = i4::from(v);
            self.sec_counter = 0.0;
        }

        self.sec_counter += 1.0 / sample_rate;
        self.last_signal
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
    fn set_pitch(&mut self, _note: i16) {}

    fn set_velocity(&mut self, velocity: f32) {
        self.velocity = velocity;
    }
}
