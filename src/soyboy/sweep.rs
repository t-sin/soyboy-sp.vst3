use std::convert::TryFrom;

use crate::{
    common::f64_utils,
    soyboy::{
        event::{Event, Triggered},
        parameters::{ParameterDef, Parametric, SoyBoyParameter},
        types::AudioProcessor,
    },
};

#[derive(Debug, Copy, Clone)]
pub enum SweepType {
    None = 0,
    Up,
    Down,
    Triangle,
}

impl TryFrom<u32> for SweepType {
    type Error = ();

    fn try_from(id: u32) -> Result<Self, Self::Error> {
        if id == SweepType::None as u32 {
            Ok(SweepType::None)
        } else if id == SweepType::Up as u32 {
            Ok(SweepType::Up)
        } else if id == SweepType::Down as u32 {
            Ok(SweepType::Down)
        } else if id == SweepType::Triangle as u32 {
            Ok(SweepType::Triangle)
        } else {
            Err(())
        }
    }
}

pub struct SweepOscillator {
    shadow_freq: f64,
    sweep_timer_sec: f64,

    clipped: bool,
    sweep_type: SweepType,
    sweep_amount: f64,
    sweep_period: f64,
}

impl SweepOscillator {
    pub fn new() -> Self {
        SweepOscillator {
            shadow_freq: 0.0,
            sweep_timer_sec: 0.0,

            clipped: false,
            sweep_type: SweepType::None,
            sweep_amount: 0.0,
            sweep_period: 0.0,
        }
    }

    fn check_frequency_clip(&mut self) {
        if self.shadow_freq < 10.0 || self.shadow_freq > 10000.0 {
            self.clipped = true;
        }
    }

    pub fn is_clipped(&self) -> bool {
        self.clipped
    }
}

const SWEEP_TIMER_FREQUENCY: f64 = 128.0;

impl AudioProcessor<f64> for SweepOscillator {
    fn process(&mut self, sample_rate: f64) -> f64 {
        if self.sweep_amount == 0.0 || self.sweep_period == 0.0 {
            return 0.0;
        }

        self.sweep_timer_sec += 1.0 / sample_rate;

        let sweep_timer_interval = 1.0 / SWEEP_TIMER_FREQUENCY;
        let fmod = self.shadow_freq * 2.0f64.powf(self.sweep_amount - 8.1);
        let fmod = f64_utils::normalize(fmod);

        match self.sweep_type {
            SweepType::None => 0.0,
            SweepType::Up => {
                let interval = sweep_timer_interval * self.sweep_period;

                if self.sweep_timer_sec > interval {
                    self.sweep_timer_sec = 0.0;
                    self.shadow_freq += fmod;

                    self.check_frequency_clip();
                    fmod
                } else {
                    0.0
                }
            }
            SweepType::Down => {
                let interval = sweep_timer_interval * self.sweep_period;

                if self.sweep_timer_sec > interval {
                    self.sweep_timer_sec = 0.0;
                    self.shadow_freq -= fmod;

                    self.check_frequency_clip();
                    -fmod
                } else {
                    0.0
                }
            }
            SweepType::Triangle => {
                let quater_period = self.sweep_period * 1.0 / SWEEP_TIMER_FREQUENCY;
                let fmod = 2.0f64.powf(self.sweep_amount - 8.1) / self.sweep_period;
                let fmod = f64_utils::normalize(fmod);

                self.check_frequency_clip();

                if self.sweep_timer_sec < quater_period {
                    fmod
                } else if self.sweep_timer_sec < quater_period * 3.0 {
                    -fmod
                } else if self.sweep_timer_sec >= quater_period * 4.0 {
                    self.sweep_timer_sec = 0.0;
                    fmod
                } else {
                    fmod
                }
            }
        }
    }

    fn set_freq(&mut self, _freq: f64) {}
}

impl Triggered for SweepOscillator {
    fn trigger(&mut self, event: &Event) {
        match event {
            Event::SweepReset { freq } => {
                self.shadow_freq = *freq;
                self.sweep_timer_sec = 0.0;
                self.clipped = false;
            }
            _ => (),
        }
    }
}

impl Parametric<SoyBoyParameter> for SweepOscillator {
    fn set_param(&mut self, param: &SoyBoyParameter, _param_def: &ParameterDef, value: f64) {
        match param {
            SoyBoyParameter::SweepType => {
                if let Ok(sweep_type) = SweepType::try_from(value as u32) {
                    self.clipped = false;
                    self.sweep_timer_sec = 0.0;
                    self.sweep_type = sweep_type;
                }
            }
            SoyBoyParameter::SweepAmount => {
                self.sweep_amount = value;
            }
            SoyBoyParameter::SweepPeriod => {
                self.sweep_period = value;
            }
            _ => (),
        }
    }

    fn get_param(&self, param: &SoyBoyParameter) -> f64 {
        match param {
            SoyBoyParameter::SweepType => (self.sweep_type as u32).into(),
            SoyBoyParameter::SweepAmount => self.sweep_amount,
            SoyBoyParameter::SweepPeriod => self.sweep_period,
            _ => 0.0,
        }
    }
}
