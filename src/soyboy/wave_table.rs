use rand::prelude::*;

use crate::{
    common::{constants, f64_utils, i4},
    soyboy::{
        event::{Event, Triggered},
        parameters::{Parametric, SoyBoyParameter},
        types::AudioProcessor,
    },
};

pub struct WaveTableOscillator {
    phase: f64,
    pitch: f64,
    pub freq: f64,

    table: [i4; constants::WAVETABLE_SIZE],
}

impl WaveTableOscillator {
    pub fn new() -> Self {
        let mut osc = WaveTableOscillator {
            phase: 0.0,
            freq: 0.0,
            pitch: 0.0,

            table: [i4::from(0.0); constants::WAVETABLE_SIZE],
        };

        osc.initialize_table();
        osc
    }

    fn randomize_table(&mut self) {
        for e in self.table.iter_mut() {
            let v = (random::<f64>() * i4::MAX as f64) as u8;
            *e = i4::from(v);
        }
    }

    fn initialize_table(&mut self) {
        let mut phase: f64 = 0.0;
        for e in self.table.iter_mut() {
            let v = (phase * 2.0 * std::f64::consts::PI).sin();
            let v = f64_utils::normalize(v);
            let v = ((v + 1.0) * i4::SIGNED_MIN.abs() as f64) as u8;
            *e = i4::from(v);
            phase += 1.0 / constants::WAVETABLE_SIZE as f64;
        }
    }

    pub fn get_wavetable(&self) -> [i4; constants::WAVETABLE_SIZE] {
        let mut table: [i4; constants::WAVETABLE_SIZE] = [i4::from(0i8); constants::WAVETABLE_SIZE];

        for (i, v) in table.iter_mut().enumerate() {
            *v = self.table[i];
        }

        table
    }

    pub fn set_wavetable(&mut self, wavetable: &[i4; constants::WAVETABLE_SIZE]) {
        self.table = wavetable.clone();
    }
}

impl Triggered for WaveTableOscillator {
    fn trigger(&mut self, event: &Event) {
        match event {
            Event::PitchBend { ratio } => {
                self.pitch = *ratio;
            }
            Event::SetWaveTable { idx, value } => {
                let idx = *idx;
                if idx < constants::WAVETABLE_SIZE {
                    println!("Wavetable::set_sample({}, {:?})", idx, value);
                    self.table[idx] = *value;
                }
            }
            Event::ResetWaveTableAsSine => {
                self.initialize_table();
            }
            Event::ResetWaveTableAtRandom => {
                self.randomize_table();
            }
            _ => (),
        }
    }
}

impl Parametric<SoyBoyParameter> for WaveTableOscillator {
    fn set_param(&mut self, _param: &SoyBoyParameter, _value: f64) {}

    fn get_param(&self, _param: &SoyBoyParameter) -> f64 {
        0.0
    }
}

impl AudioProcessor<i4> for WaveTableOscillator {
    fn process(&mut self, sample_rate: f64) -> i4 {
        let v = self.table[self.phase as usize];

        let wt_size = constants::WAVETABLE_SIZE as f64;
        let phase_diff = ((self.freq * self.pitch) / sample_rate) * wt_size;
        self.phase = (self.phase + phase_diff) % wt_size;

        v
    }

    fn set_freq(&mut self, freq: f64) {
        self.freq = freq;
    }
}
