use std::convert::TryFrom;

use crate::soyboy::{
    parameters::{Parameter, Parametric},
    types::{i4, AudioProcessor, Oscillator},
    utils::{frequency_from_note_number, level_from_velocity, pulse},
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
    fn new() -> Self {
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
        if self.shadow_freq > 10000.0 {
            self.clipped = true;
            self.shadow_freq = 0.0;
        } else if self.shadow_freq < 10.0 {
            self.clipped = true;
            self.shadow_freq = 0.0;
        }
    }
}

impl AudioProcessor<f64> for SweepOscillator {
    fn process(&mut self, sample_rate: f64) -> f64 {
        if self.sweep_amount == 0.0 || self.sweep_period == 0.0 {
            return 0.0;
        }

        self.sweep_timer_sec += 1.0 / sample_rate;

        let sweep_timer_interval = 1.0 / SWEEP_TIMER_FREQUENCY;
        let fmod = self.shadow_freq * 2.0f64.powf(self.sweep_amount - 8.1);

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
}

impl Parametric<Parameter> for SweepOscillator {
    fn set_param(&mut self, param: &Parameter, value: f64) {
        match param {
            Parameter::OscSqSweepType => {
                self.sweep_timer_sec = 0.0;
                if let Ok(sweep_type) = SweepType::try_from(value as u32) {
                    self.sweep_type = sweep_type;
                } else {
                    ()
                }
            }
            Parameter::OscSqSweepAmount => {
                self.sweep_amount = value;
            }
            Parameter::OscSqSweepPeriod => {
                self.sweep_period = value;
            }
            _ => (),
        }
    }

    fn get_param(&self, param: &Parameter) -> f64 {
        match param {
            Parameter::OscSqSweepType => (self.sweep_type as u32).into(),
            Parameter::OscSqSweepAmount => self.sweep_amount,
            Parameter::OscSqSweepPeriod => self.sweep_period,
            _ => 0.0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SquareWaveDuty {
    Ratio12_5 = 0,
    Ratio25,
    Ratio50,
}

impl TryFrom<u32> for SquareWaveDuty {
    type Error = ();

    fn try_from(id: u32) -> Result<Self, Self::Error> {
        if id == SquareWaveDuty::Ratio12_5 as u32 {
            Ok(SquareWaveDuty::Ratio12_5)
        } else if id == SquareWaveDuty::Ratio25 as u32 {
            Ok(SquareWaveDuty::Ratio25)
        } else if id == SquareWaveDuty::Ratio50 as u32 {
            Ok(SquareWaveDuty::Ratio50)
        } else {
            Err(())
        }
    }
}

impl SquareWaveDuty {
    fn to_ratio(&self) -> f64 {
        match self {
            SquareWaveDuty::Ratio12_5 => 0.125,
            SquareWaveDuty::Ratio25 => 0.25,
            SquareWaveDuty::Ratio50 => 0.5,
        }
    }
}

pub struct SquareWaveOscillator {
    phase: f64,
    velocity: f64,
    freq: f64,
    sweep: SweepOscillator,

    duty: SquareWaveDuty,
}

impl SquareWaveOscillator {
    pub fn new() -> Self {
        SquareWaveOscillator {
            phase: 0.0,
            velocity: 0.0,
            freq: 440.0,

            duty: SquareWaveDuty::Ratio50,
            sweep: SweepOscillator::new(),
        }
    }

    pub fn set_duty(&mut self, duty: SquareWaveDuty) {
        self.duty = duty;
    }
}

const SWEEP_TIMER_FREQUENCY: f64 = 128.0;

impl AudioProcessor<i4> for SquareWaveOscillator {
    fn process(&mut self, sample_rate: f64) -> i4 {
        let signal = if self.freq == 0.0 {
            i4::ZERO
        } else {
            let signal = pulse(self.phase, self.duty.to_ratio());
            i4::from((signal.to_i8() as f64 * level_from_velocity(self.velocity)) as i8)
        };

        if self.sweep.clipped {
            self.freq = 0.0;
        } else {
            self.freq += self.sweep.process(sample_rate);
        }

        let phase_diff = self.freq / sample_rate;
        self.phase += phase_diff;

        signal
    }
}

impl Parametric<Parameter> for SquareWaveOscillator {
    fn set_param(&mut self, param: &Parameter, value: f64) {
        match param {
            Parameter::OscSqDuty => {
                if let Ok(ratio) = SquareWaveDuty::try_from(value as u32) {
                    self.set_duty(ratio);
                } else {
                    ()
                }
            }
            Parameter::OscSqSweepType => self.sweep.set_param(param, value),
            Parameter::OscSqSweepAmount => self.sweep.set_param(param, value),
            Parameter::OscSqSweepPeriod => self.sweep.set_param(param, value),
            _ => (),
        }
    }

    fn get_param(&self, param: &Parameter) -> f64 {
        match param {
            Parameter::OscSqDuty => (self.duty as u32).into(),
            Parameter::OscSqSweepType => self.sweep.get_param(param),
            Parameter::OscSqSweepAmount => self.sweep.get_param(param),
            Parameter::OscSqSweepPeriod => self.sweep.get_param(param),
            _ => 0.0,
        }
    }
}

impl Oscillator for SquareWaveOscillator {
    fn set_pitch(&mut self, note: i16) {
        self.freq = frequency_from_note_number(note);
        self.sweep.shadow_freq = self.freq;
        self.sweep.clipped = false;
    }

    fn set_velocity(&mut self, velocity: f32) {
        self.velocity = velocity as f64;
    }
}
