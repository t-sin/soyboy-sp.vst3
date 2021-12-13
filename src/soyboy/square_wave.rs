use std::convert::TryFrom;

use crate::soyboy::{
    event::{Event, Triggered},
    parameters::{Parameter, Parametric},
    types::{i4, AudioProcessor},
    utils::pulse,
};

#[derive(Debug, Copy, Clone)]
pub enum SquareWaveDuty {
    Ratio12_5 = 0,
    Ratio25,
    Ratio50,
    Ratio75,
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
        } else if id == SquareWaveDuty::Ratio75 as u32 {
            Ok(SquareWaveDuty::Ratio75)
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
            SquareWaveDuty::Ratio75 => 0.75,
        }
    }
}

pub struct SquareWaveOscillator {
    phase: f64,
    pub freq: f64,

    duty: SquareWaveDuty,
    pitch: f64,
}

impl SquareWaveOscillator {
    pub fn new() -> Self {
        SquareWaveOscillator {
            phase: 0.0,
            freq: 0.0,

            duty: SquareWaveDuty::Ratio50,
            pitch: 0.0,
        }
    }

    pub fn set_duty(&mut self, duty: SquareWaveDuty) {
        self.duty = duty;
    }
}

impl Triggered for SquareWaveOscillator {
    fn trigger(&mut self, event: &Event) {
        match event {
            Event::PitchBend { ratio } => {
                self.pitch = *ratio;
            }
            _ => (),
        }
    }
}

impl AudioProcessor<i4> for SquareWaveOscillator {
    fn process(&mut self, sample_rate: f64) -> i4 {
        let signal = if self.freq == 0.0 {
            i4::from(i4::zero())
        } else {
            pulse(self.phase, self.duty.to_ratio())
        };

        let phase_diff = (self.freq * self.pitch) / sample_rate;
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
            _ => (),
        }
    }

    fn get_param(&self, param: &Parameter) -> f64 {
        match param {
            Parameter::OscSqDuty => (self.duty as u32).into(),
            _ => 0.0,
        }
    }
}
