use std::convert::TryFrom;

use crate::soyboy::{
    parameters::{Parameter, Parametric},
    types::{i4, AudioProcessor, Oscillator},
    utils::{frequency_from_note_number, level_from_velocity, pulse},
};

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

pub struct SquareWaveOscillator {
    phase: f64,
    velocity: f64,
    freq: f64,

    duty: SquareWaveDuty,
    sweep_type: SweepType,
    sweep_speed: f64,
}

impl SquareWaveOscillator {
    pub fn new() -> Self {
        SquareWaveOscillator {
            phase: 0.0,
            velocity: 0.0,
            freq: 440.0,
            duty: SquareWaveDuty::Ratio50,
            sweep_type: SweepType::None,
            sweep_speed: 0.0,
        }
    }

    pub fn set_duty(&mut self, duty: SquareWaveDuty) {
        self.duty = duty;
    }
}

impl AudioProcessor<i4> for SquareWaveOscillator {
    fn process(&mut self, sample_rate: f64) -> i4 {
        let phase_diff = self.freq / sample_rate;
        let v = pulse(self.phase, self.duty.to_ratio());

        self.phase += phase_diff;
        i4::from((v.to_f64() * level_from_velocity(self.velocity)) as i8)
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
            Parameter::OscSqSweepType => {
                if let Ok(sweep_type) = SweepType::try_from(value as u32) {
                    self.sweep_type = sweep_type;
                } else {
                    ()
                }
            }
            Parameter::OscSqSweepSpeed => {
                self.sweep_speed = value;
            }
            _ => (),
        }
    }

    fn get_param(&self, param: &Parameter) -> f64 {
        match param {
            Parameter::OscSqDuty => (self.duty as u32).into(),
            Parameter::OscSqSweepType => (self.sweep_type as u32).into(),
            Parameter::OscSqSweepSpeed => self.sweep_speed,
            _ => 0.0,
        }
    }
}

impl Oscillator for SquareWaveOscillator {
    fn set_pitch(&mut self, note: i16) {
        self.freq = frequency_from_note_number(note);
    }

    fn set_velocity(&mut self, velocity: f32) {
        self.velocity = velocity as f64;
    }
}
