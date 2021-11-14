use std::convert::TryFrom;

use crate::soyboy::{
    parameters::{Parameter, Parametric},
    types::{i4, AudioProcessor, Oscillator},
    utils::{frequency_from_note_number, pulse},
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

pub struct SquareWaveOscillator {
    phase: f64,
    velocity: f32,
    freq: f64,
    duty: SquareWaveDuty,
}

impl SquareWaveOscillator {
    pub fn new() -> Self {
        SquareWaveOscillator {
            phase: 0.0,
            velocity: 0.0,
            freq: 440.0,
            duty: SquareWaveDuty::Ratio50,
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
        i4::from((v.to_f64() * self.velocity as f64) as i8)
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

const NOTE_NUMBER_OF_440_HZ: u16 = 69;

impl Oscillator for SquareWaveOscillator {
    /// https://steinbergmedia.github.io/vst3_doc/vstinterfaces/structSteinberg_1_1Vst_1_1NoteOnEvent.html の pitch の項目
    fn set_pitch(&mut self, note: i16) {
        self.freq = frequency_from_note_number(note as u16, NOTE_NUMBER_OF_440_HZ);
    }

    fn set_velocity(&mut self, velocity: f32) {
        self.velocity = velocity;
    }
}
