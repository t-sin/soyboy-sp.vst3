use crate::soyboy::{
    event::{Event, Triggered},
    parameters::{Parameter, Parametric},
    types::{i4, AudioProcessor},
    utils::level_from_velocity,
};

pub struct WaveTableOscillator {
    phase: f64,
    freq: f64,
    velocity: f64,
}

impl WaveTableOscillator {
    pub fn new() -> Self {
        WaveTableOscillator {
            phase: 0.0,
            freq: 0.0,
            velocity: 0.0,
        }
    }
}

impl Triggered for WaveTableOscillator {
    fn trigger(&mut self, _event: &Event) {}
}

impl Parametric<Parameter> for WaveTableOscillator {
    fn set_param(&mut self, _param: &Parameter, _value: f64) {}
    fn get_param(&self, _param: &Parameter) -> f64 {
        0.0
    }
}

impl AudioProcessor<i4> for WaveTableOscillator {
    fn process(&mut self, sample_rate: f64) -> i4 {
        let signal = i4::from((i4::ZERO.to_i8() as f64 * level_from_velocity(self.velocity)) as i8);

        let phase_diff = self.freq / sample_rate;
        self.phase += phase_diff;

        signal
    }
}
