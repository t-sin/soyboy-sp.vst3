use crate::soyboy::{
    event::{Event, Triggered},
    parameters::{Parameter, Parametric},
    types::AudioProcessor,
    utils::level,
    voice::VoiceUnit,
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
}

impl Triggered for SoyBoy {
    fn trigger(&mut self, event: &Event) {
        self.voice.trigger(event);
    }
}

impl Parametric<Parameter> for SoyBoy {
    fn set_param(&mut self, param: &Parameter, value: f64) {
        match param {
            Parameter::MasterVolume => self.master_volume = value,
            param => self.voice.set_param(param, value),
        }
    }

    fn get_param(&self, param: &Parameter) -> f64 {
        match param {
            Parameter::MasterVolume => self.master_volume,
            param => self.voice.get_param(param),
        }
    }
}
impl AudioProcessor<Signal> for SoyBoy {
    fn process(&mut self, sample_rate: f64) -> Signal {
        let v = self.voice.process(sample_rate) * level(self.master_volume);
        (v, v)
    }
}
