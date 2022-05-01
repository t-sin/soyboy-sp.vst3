mod dac;
mod envelope_generator;
mod noise;
mod square_wave;
mod stutter;
mod sweep;
mod types;
mod utils;
mod voice;
mod wave_table;

pub mod event;
pub mod parameters;

pub use parameters::*;
pub use types::*;

use crate::{
    common::{constants, i4},
    soyboy::{
        event::{Event, Triggered},
        utils::level,
        voice::VoiceUnit,
    },
};

pub type Signal = (f64, f64);

pub struct SoyBoy {
    voice: VoiceUnit,

    master_volume: f64,
}

impl SoyBoy {
    pub fn new() -> Self {
        Self {
            voice: VoiceUnit::new(),

            master_volume: 1.0,
        }
    }

    pub fn get_wavetable(&self) -> [i4; constants::WAVETABLE_SIZE] {
        self.voice.get_wavetable()
    }
}

impl Triggered for SoyBoy {
    fn trigger(&mut self, event: &Event) {
        self.voice.trigger(event);
    }
}

impl Parametric<SoyBoyParameter> for SoyBoy {
    fn set_param(&mut self, param: &SoyBoyParameter, value: f64) {
        match param {
            SoyBoyParameter::MasterVolume => self.master_volume = value,
            param => self.voice.set_param(param, value),
        }
    }

    fn get_param(&self, param: &SoyBoyParameter) -> f64 {
        match param {
            SoyBoyParameter::MasterVolume => self.master_volume,
            param => self.voice.get_param(param),
        }
    }
}
impl AudioProcessor<Signal> for SoyBoy {
    fn process(&mut self, sample_rate: f64) -> Signal {
        let v = self.voice.process(sample_rate) * level(self.master_volume);
        (v, v)
    }

    fn set_freq(&mut self, _freq: f64) {}
}
