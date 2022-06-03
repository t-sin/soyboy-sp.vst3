use std::convert::TryFrom;

use crate::{
    common::f64_utils,
    soyboy::{
        event::{Event, Triggered},
        parameters::{ParameterDef, Parametric, SoyBoyParameter},
        types::AudioProcessor,
        utils::{discrete_loudness, level_from_velocity, linear},
    },
};

#[derive(Debug, Copy, Clone)]
enum StartTiming {
    NoteOff = 0,
    NoteOn,
}

impl TryFrom<u32> for StartTiming {
    type Error = ();

    fn try_from(id: u32) -> Result<Self, Self::Error> {
        if id == StartTiming::NoteOn as u32 {
            Ok(StartTiming::NoteOn)
        } else if id == StartTiming::NoteOff as u32 {
            Ok(StartTiming::NoteOff)
        } else {
            Err(())
        }
    }
}

#[derive(Debug)]
pub enum EnvelopeState {
    Attack,
    Decay,
    Sustain,
    Release,
    Off,
}

pub struct EnvelopeGenerator {
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
    stutter_time: f64,
    stutter_depth: f64,
    stutter_when: StartTiming,

    velocity: f64,
    note: u16,
    state: EnvelopeState,
    elapsed_samples: u64,
    last_value: f64,
    last_state_value: f64,

    stuttering: bool,
    stutter_velocity: f64,
}

impl EnvelopeGenerator {
    pub fn new() -> EnvelopeGenerator {
        EnvelopeGenerator {
            attack: 0.05,
            decay: 0.05,
            sustain: 0.3,
            release: 0.1,
            stutter_time: 0.1,
            stutter_depth: 0.0,
            stutter_when: StartTiming::NoteOn,

            velocity: 0.0,
            note: 0,
            state: EnvelopeState::Off,
            elapsed_samples: 1,
            last_value: 0.0,
            last_state_value: 0.0,

            stuttering: false,
            stutter_velocity: 0.0,
        }
    }

    pub fn same_note(&self, note: u16) -> bool {
        self.note == note
    }

    pub fn assignable(&self, note: u16) -> bool {
        let same_note = self.same_note(note);
        let silent = match self.state {
            EnvelopeState::Release => true,
            EnvelopeState::Off => true,
            _ => false,
        };

        same_note || silent || self.stuttering
    }

    pub fn set_state(&mut self, state: EnvelopeState) {
        match self.state {
            EnvelopeState::Attack => self.last_state_value = self.last_value,
            EnvelopeState::Decay => self.last_state_value = self.last_value,
            EnvelopeState::Sustain => self.last_state_value = self.last_value,
            _ => (),
        }
        self.state = state;
        self.elapsed_samples = 0;
    }

    fn update_state(&mut self, s: f64) {
        match self.state {
            EnvelopeState::Attack => {
                if s > self.attack {
                    self.set_state(EnvelopeState::Decay);
                    self.last_state_value = 1.0;
                }
            }
            EnvelopeState::Decay => {
                if s > self.decay {
                    self.set_state(EnvelopeState::Sustain);
                }
            }
            EnvelopeState::Sustain => (),
            EnvelopeState::Release => {
                if s > self.release {
                    self.set_state(EnvelopeState::Off);
                }
            }
            EnvelopeState::Off => (),
        };
    }

    fn calculate(&mut self, s: f64) -> f64 {
        match self.state {
            EnvelopeState::Attack => linear(s, 1.0 / self.attack),
            EnvelopeState::Decay => {
                let sustain = f64_utils::normalize(self.sustain);
                let max = self.last_state_value - sustain;
                self.last_state_value - max * linear(s, 1.0 / self.decay)
            }
            EnvelopeState::Sustain => f64_utils::normalize(self.sustain),
            EnvelopeState::Release => {
                let max = self.last_state_value;
                max - max * linear(s, 1.0 / self.release)
            }
            EnvelopeState::Off => 0.0,
        }
    }

    fn stutter(&mut self, elapsed_sec: f64) {
        if self.stutter_depth > 0.0 && elapsed_sec > self.stutter_time {
            self.stuttering = true;
            self.stutter_velocity -= 1.0 - self.stutter_depth / 100.0;

            if self.stutter_velocity > 0.05 {
                self.set_state(EnvelopeState::Attack);
            } else {
                self.stutter_velocity = 0.0;
                self.stuttering = false;
            }
        }
    }

    fn start_stutter(&mut self) {
        if let StartTiming::NoteOn = self.stutter_when {
            self.stuttering = true;
            self.stutter_velocity = 1.0;
        } else {
            self.stuttering = false;
        }
    }
}

impl AudioProcessor<f64> for EnvelopeGenerator {
    fn process(&mut self, sample_rate: f64) -> f64 {
        let s = self.elapsed_samples as f64 / sample_rate;

        self.stutter(s);
        self.update_state(s);
        let v = self.calculate(s);
        let v = f64_utils::normalize(v);
        self.last_value = v;
        self.elapsed_samples += 1;

        discrete_loudness(v * self.stutter_velocity * level_from_velocity(self.velocity))
    }

    fn set_freq(&mut self, _freq: f64) {}
}

impl Triggered for EnvelopeGenerator {
    fn trigger(&mut self, event: &Event) {
        match event {
            Event::NoteOn { note, velocity } => {
                self.note = *note;
                self.set_state(EnvelopeState::Attack);
                self.velocity = *velocity;
                self.start_stutter();
            }
            Event::NoteOff { note } => {
                if *note == self.note {
                    self.set_state(EnvelopeState::Release);
                    self.start_stutter();
                }
            }
            _ => (),
        }
    }
}

impl Parametric<SoyBoyParameter> for EnvelopeGenerator {
    fn set_param(&mut self, param: &SoyBoyParameter, _param_def: &ParameterDef, value: f64) {
        match param {
            SoyBoyParameter::EgAttack => self.attack = value,
            SoyBoyParameter::EgDecay => self.decay = value,
            SoyBoyParameter::EgSustain => self.sustain = value,
            SoyBoyParameter::EgRelease => self.release = value,
            SoyBoyParameter::StutterTime => self.stutter_time = value,
            SoyBoyParameter::StutterDepth => self.stutter_depth = value,
            SoyBoyParameter::StutterWhen => {
                if let Ok(when) = StartTiming::try_from(value as u32) {
                    self.stutter_when = when;
                }
            }
            _ => (),
        }
    }

    fn get_param(&self, param: &SoyBoyParameter) -> f64 {
        match param {
            SoyBoyParameter::EgAttack => self.attack,
            SoyBoyParameter::EgDecay => self.decay,
            SoyBoyParameter::EgSustain => self.sustain,
            SoyBoyParameter::EgRelease => self.release,
            SoyBoyParameter::StutterTime => self.stutter_time,
            SoyBoyParameter::StutterDepth => self.stutter_depth,
            SoyBoyParameter::StutterWhen => (self.stutter_when as u32).into(),
            _ => 0.0,
        }
    }
}
