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
    voices: Vec<VoiceUnit>,

    num_voices: usize,
    master_volume: f64,
}

impl SoyBoy {
    pub fn new() -> Self {
        let mut voices = Vec::new();
        for _ in 0..constants::MAX_NUMBER_OF_VOICES {
            voices.push(VoiceUnit::new());
        }

        Self {
            voices,

            num_voices: 4,
            master_volume: 1.0,
        }
    }

    fn get_voices(&mut self) -> &mut [VoiceUnit] {
        &mut self.voices[0..self.num_voices]
    }

    pub fn get_wavetable(&self) -> [i4; constants::WAVETABLE_SIZE] {
        self.voices[0].get_wavetable()
    }

    pub fn set_wavetable(&mut self, wavetable: &[i4; constants::WAVETABLE_SIZE]) {
        self.get_voices()
            .iter_mut()
            .for_each(|v| v.set_wavetable(wavetable));
    }
}

impl Triggered for SoyBoy {
    fn trigger(&mut self, event: &Event) {
        match event {
            Event::NoteOn { note, velocity: _ } => {
                if let Some(voice) = self.get_voices().iter_mut().find(|v| v.assignable(*note)) {
                    voice.trigger(event);
                }
            }
            Event::NoteOff { note } => {
                if let Some(voice) = self.voices.iter_mut().find(|v| v.same_note(*note)) {
                    voice.trigger(event);
                }
            }
            event => self.voices.iter_mut().for_each(|v| v.trigger(event)),
        }
    }
}

impl Parametric<SoyBoyParameter> for SoyBoy {
    fn set_param(&mut self, param: &SoyBoyParameter, value: f64) {
        match param {
            SoyBoyParameter::MasterVolume => self.master_volume = value,
            SoyBoyParameter::NumVoices => self.num_voices = value as usize,
            param => self
                .voices
                .iter_mut()
                .for_each(|v| v.set_param(param, value)),
        }
    }

    fn get_param(&self, param: &SoyBoyParameter) -> f64 {
        match param {
            SoyBoyParameter::MasterVolume => self.master_volume,
            SoyBoyParameter::NumVoices => self.num_voices as f64,
            param => self.voices[0].get_param(param),
        }
    }
}
impl AudioProcessor<Signal> for SoyBoy {
    fn process(&mut self, sample_rate: f64) -> Signal {
        let mut v = 0.0;

        for voice in self.get_voices().iter_mut() {
            v += voice.process(sample_rate);
        }

        let v = v * level(self.master_volume);

        (v, v)
    }

    fn set_freq(&mut self, _freq: f64) {}
}
