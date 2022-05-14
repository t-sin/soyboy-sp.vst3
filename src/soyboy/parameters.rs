use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;
use std::convert::TryFrom;

use crate::soyboy::utils;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum SoyBoyParameter {
    // global parameter
    MasterVolume = 0,
    PitchBend,
    Detune,
    OscillatorType,
    NumVoices,
    // frequency sweep
    SweepType,
    SweepAmount,
    SweepPeriod,
    // note stutter
    StutterTime,
    StutterDepth,
    // envelope generator
    EgAttack,
    EgDecay,
    EgSustain,
    EgRelease,
    // square wave oscillator
    OscSqDuty,
    // noise oscillator
    OscNsInterval,
    // hidden: DAC
    DacFreq,
    DacQ,
}

impl TryFrom<u32> for SoyBoyParameter {
    type Error = ();

    fn try_from(id: u32) -> Result<Self, Self::Error> {
        if id == SoyBoyParameter::MasterVolume as u32 {
            Ok(SoyBoyParameter::MasterVolume)
        } else if id == SoyBoyParameter::OscillatorType as u32 {
            Ok(SoyBoyParameter::OscillatorType)
        } else if id == SoyBoyParameter::NumVoices as u32 {
            Ok(SoyBoyParameter::NumVoices)
        } else if id == SoyBoyParameter::PitchBend as u32 {
            Ok(SoyBoyParameter::PitchBend)
        } else if id == SoyBoyParameter::Detune as u32 {
            Ok(SoyBoyParameter::Detune)
        } else if id == SoyBoyParameter::SweepType as u32 {
            Ok(SoyBoyParameter::SweepType)
        } else if id == SoyBoyParameter::SweepAmount as u32 {
            Ok(SoyBoyParameter::SweepAmount)
        } else if id == SoyBoyParameter::SweepPeriod as u32 {
            Ok(SoyBoyParameter::SweepPeriod)
        } else if id == SoyBoyParameter::StutterTime as u32 {
            Ok(SoyBoyParameter::StutterTime)
        } else if id == SoyBoyParameter::StutterDepth as u32 {
            Ok(SoyBoyParameter::StutterDepth)
        } else if id == SoyBoyParameter::EgAttack as u32 {
            Ok(SoyBoyParameter::EgAttack)
        } else if id == SoyBoyParameter::EgDecay as u32 {
            Ok(SoyBoyParameter::EgDecay)
        } else if id == SoyBoyParameter::EgSustain as u32 {
            Ok(SoyBoyParameter::EgSustain)
        } else if id == SoyBoyParameter::EgRelease as u32 {
            Ok(SoyBoyParameter::EgRelease)
        } else if id == SoyBoyParameter::OscSqDuty as u32 {
            Ok(SoyBoyParameter::OscSqDuty)
        } else if id == SoyBoyParameter::OscNsInterval as u32 {
            Ok(SoyBoyParameter::OscNsInterval)
        } else if id == SoyBoyParameter::DacFreq as u32 {
            Ok(SoyBoyParameter::DacFreq)
        } else if id == SoyBoyParameter::DacQ as u32 {
            Ok(SoyBoyParameter::DacQ)
        } else {
            Err(())
        }
    }
}

pub struct ParamIter(u32);

impl ParamIter {
    pub fn new() -> ParamIter {
        ParamIter(SoyBoyParameter::MasterVolume as u32)
    }
}

impl Iterator for ParamIter {
    type Item = SoyBoyParameter;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(p) = SoyBoyParameter::try_from(self.0) {
            self.0 += 1;
            Some(p)
        } else {
            None
        }
    }
}

impl SoyBoyParameter {
    pub fn iter() -> ParamIter {
        ParamIter::new()
    }
}

pub trait Parametric<Parameter> {
    fn set_param(&mut self, param: &Parameter, value: f64);
    fn get_param(&self, param: &Parameter) -> f64;
}

#[derive(Clone, Copy, Debug)]
pub enum ParameterType {
    NonLinear,
    Linear,
    List,
    Integer,
}

pub trait Normalizable<T> {
    fn denormalize(&self, normalized: f64) -> T;
    fn normalize(&self, plain: T) -> f64;
    fn format(&self, normalized: f64) -> String;
    fn parse(&self, string: &str) -> Option<f64>;
}

#[derive(Clone, Copy)]
pub struct NonLinearParameter {
    plain_zero: f64,
    plain_min: f64,
    plain_max: f64,
    plain_one: f64,
    factor: f64,
    diverge: bool,
}

impl Normalizable<f64> for NonLinearParameter {
    fn denormalize(&self, normalized: f64) -> f64 {
        if normalized == 0.0 {
            self.plain_zero
        } else if normalized == 1.0 {
            self.plain_one
        } else {
            let denormalizer = if self.diverge {
                utils::divergent_denormalize
            } else {
                utils::convergent_denormalize
            };
            denormalizer(normalized, self.plain_min, self.plain_max, self.factor)
        }
    }

    fn normalize(&self, plain: f64) -> f64 {
        if plain == self.plain_zero {
            0.0
        } else if plain == self.plain_one {
            1.0
        } else {
            let normalizer = if self.diverge {
                utils::divergent_normalize
            } else {
                utils::convergent_normalize
            };
            normalizer(plain, self.plain_min, self.plain_max, self.factor)
        }
    }

    fn format(&self, normalized: f64) -> String {
        format!("{:.3}", self.denormalize(normalized))
    }

    fn parse(&self, string: &str) -> Option<f64> {
        if let Some(vs) = string.split(' ').next() {
            if let Ok(v) = vs.parse() {
                Some(self.normalize(v))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
pub struct LinearParameter {
    min: f64,
    max: f64,
}

impl Normalizable<f64> for LinearParameter {
    fn denormalize(&self, normalized: f64) -> f64 {
        utils::linear_denormalize(normalized, self.min, self.max)
    }

    fn normalize(&self, plain: f64) -> f64 {
        utils::linear_normalize(plain, self.min, self.max)
    }

    fn format(&self, normalized: f64) -> String {
        format!("{:.2}", self.denormalize(normalized))
    }

    fn parse(&self, string: &str) -> Option<f64> {
        if let Some(vs) = string.split(' ').next() {
            if let Ok(v) = vs.parse() {
                Some(self.normalize(v))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
pub struct IntegerParameter {
    min: i32,
    max: i32,
}

impl Normalizable<f64> for IntegerParameter {
    fn denormalize(&self, normalized: f64) -> f64 {
        utils::linear_denormalize(normalized, self.min as f64, self.max as f64) as i64 as f64
    }

    fn normalize(&self, plain: f64) -> f64 {
        utils::linear_normalize(plain, self.min as f64, self.max as f64)
    }

    fn format(&self, normalized: f64) -> String {
        format!("{:.2}", self.denormalize(normalized))
    }

    fn parse(&self, string: &str) -> Option<f64> {
        if let Some(vs) = string.split(' ').next() {
            if let Ok(v) = vs.parse() {
                Some(self.normalize(v))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
pub struct ListParameter {
    elements: &'static [&'static str],
}

impl Normalizable<f64> for ListParameter {
    fn denormalize(&self, normalized: f64) -> f64 {
        normalized * (self.elements.len() - 1) as f64
    }

    fn normalize(&self, plain: f64) -> f64 {
        plain / (self.elements.len() - 1) as f64
    }

    fn format(&self, normalized: f64) -> String {
        if let Some(s) = self.elements.get(self.denormalize(normalized) as usize) {
            s.to_string()
        } else {
            "".to_string()
        }
    }

    fn parse(&self, string: &str) -> Option<f64> {
        if let Ok(v) = self.elements.binary_search(&string) {
            Some(self.normalize(v as f64))
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
pub union ParameterInfo {
    pub non_linear: NonLinearParameter,
    pub linear: LinearParameter,
    pub list: ListParameter,
    pub int: IntegerParameter,
}

#[derive(Clone)]
pub struct ParameterDef {
    pub r#type: ParameterType,
    pub parameter: ParameterInfo,
    pub title: String,
    pub short_title: String,
    pub unit_name: String,
    pub step_count: i32,
    pub default_value: f64,
}

impl Normalizable<f64> for ParameterDef {
    fn denormalize(&self, normalized: f64) -> f64 {
        match self.r#type {
            ParameterType::NonLinear => unsafe {
                self.parameter.non_linear.denormalize(normalized)
            },
            ParameterType::Linear => unsafe { self.parameter.linear.denormalize(normalized) },
            ParameterType::List => unsafe { self.parameter.list.denormalize(normalized) },
            ParameterType::Integer => unsafe { self.parameter.int.denormalize(normalized) },
        }
    }

    fn normalize(&self, plain: f64) -> f64 {
        match self.r#type {
            ParameterType::NonLinear => unsafe { self.parameter.non_linear.normalize(plain) },
            ParameterType::Linear => unsafe { self.parameter.linear.normalize(plain) },
            ParameterType::List => unsafe { self.parameter.list.normalize(plain) },
            ParameterType::Integer => unsafe { self.parameter.int.normalize(plain) },
        }
    }

    fn format(&self, normalized: f64) -> String {
        let s = match self.r#type {
            ParameterType::NonLinear => unsafe { self.parameter.non_linear.format(normalized) },
            ParameterType::Linear => unsafe { self.parameter.linear.format(normalized) },
            ParameterType::List => unsafe { self.parameter.list.format(normalized) },
            ParameterType::Integer => unsafe { self.parameter.int.format(normalized) },
        };
        format!("{} {}", s, self.unit_name)
    }

    fn parse(&self, string: &str) -> Option<f64> {
        match self.r#type {
            ParameterType::NonLinear => unsafe { self.parameter.non_linear.parse(string) },
            ParameterType::Linear => unsafe { self.parameter.linear.parse(string) },
            ParameterType::List => unsafe { self.parameter.list.parse(string) },
            ParameterType::Integer => unsafe { self.parameter.int.parse(string) },
        }
    }
}

fn make_global_parameters(params: &mut HashMap<SoyBoyParameter, ParameterDef>) {
    static GLOBAL_MASTER_VOLUME: NonLinearParameter = NonLinearParameter {
        plain_zero: -f64::INFINITY,
        plain_min: -110.0,
        plain_max: 6.0,
        plain_one: 6.0,
        factor: 10.0,
        diverge: false,
    };
    params.insert(
        SoyBoyParameter::MasterVolume,
        ParameterDef {
            r#type: ParameterType::NonLinear,
            parameter: ParameterInfo {
                non_linear: GLOBAL_MASTER_VOLUME,
            },
            title: "Master Volume".to_string(),
            short_title: "Volume".to_string(),
            unit_name: "dB".to_string(),
            step_count: 0,
            default_value: -4.0,
        },
    );

    static GLOBAL_DETUNE: IntegerParameter = IntegerParameter {
        min: -200,
        max: 200,
    };
    params.insert(
        SoyBoyParameter::Detune,
        ParameterDef {
            r#type: ParameterType::Integer,
            parameter: ParameterInfo { int: GLOBAL_DETUNE },
            title: "Detune".to_string(),
            short_title: "Detune".to_string(),
            unit_name: "cent".to_string(),
            step_count: GLOBAL_DETUNE.max.abs() + GLOBAL_DETUNE.min.abs(),
            default_value: 0.0,
        },
    );
    static GLOBAL_PITCH: IntegerParameter = IntegerParameter {
        min: -4800,
        max: 4800,
    };
    params.insert(
        SoyBoyParameter::PitchBend,
        ParameterDef {
            r#type: ParameterType::Integer,
            parameter: ParameterInfo { int: GLOBAL_PITCH },
            title: "Pitch".to_string(),
            short_title: "Pitch".to_string(),
            unit_name: "cent".to_string(),
            step_count: GLOBAL_PITCH.max.abs() + GLOBAL_PITCH.min.abs(),
            default_value: 0.0,
        },
    );

    static SELECTED_OSCILLATOR_LIST: [&str; 3] = ["Square", "Noise", "Wavetable"];
    static SELECTED_OSC: ListParameter = ListParameter {
        elements: &SELECTED_OSCILLATOR_LIST,
    };
    params.insert(
        SoyBoyParameter::OscillatorType,
        ParameterDef {
            r#type: ParameterType::List,
            parameter: ParameterInfo { list: SELECTED_OSC },
            title: "Osc type".to_string(),
            short_title: "Osc type".to_string(),
            unit_name: "".to_string(),
            step_count: (SELECTED_OSC.denormalize(1.0)) as i32,
            default_value: 0.0,
        },
    );

    static NUM_VOICES: IntegerParameter = IntegerParameter { min: 1, max: 8 };
    params.insert(
        SoyBoyParameter::NumVoices,
        ParameterDef {
            r#type: ParameterType::Integer,
            parameter: ParameterInfo { int: NUM_VOICES },
            title: "Num of voices".to_string(),
            short_title: "Voice num".to_string(),
            unit_name: "".to_string(),
            step_count: NUM_VOICES.max - NUM_VOICES.min,
            default_value: 1.0,
        },
    );

    static SWEEP_TYPE_LIST: [&str; 4] = ["None", "Up", "Down", "Tri"];
    static SWEEP_TYPE: ListParameter = ListParameter {
        elements: &SWEEP_TYPE_LIST,
    };
    params.insert(
        SoyBoyParameter::SweepType,
        ParameterDef {
            r#type: ParameterType::List,
            parameter: ParameterInfo { list: SWEEP_TYPE },
            title: "Sweep Type".to_string(),
            short_title: "Sweep Type".to_string(),
            unit_name: "".to_string(),
            step_count: (SWEEP_TYPE.denormalize(1.0)) as i32,
            default_value: 0.0,
        },
    );
    static SWEEP_AMOUNT: IntegerParameter = IntegerParameter { min: 0, max: 8 };
    params.insert(
        SoyBoyParameter::SweepAmount,
        ParameterDef {
            r#type: ParameterType::Integer,
            parameter: ParameterInfo { int: SWEEP_AMOUNT },
            title: "Sweep Amount".to_string(),
            short_title: "Sweep Amount".to_string(),
            unit_name: "".to_string(),
            step_count: SWEEP_AMOUNT.max - SWEEP_AMOUNT.min,
            default_value: 0.0,
        },
    );
    static SWEEP_PERIOD: IntegerParameter = IntegerParameter { min: 0, max: 8 };
    params.insert(
        SoyBoyParameter::SweepPeriod,
        ParameterDef {
            r#type: ParameterType::Integer,
            parameter: ParameterInfo { int: SWEEP_PERIOD },
            title: "Sweep period".to_string(),
            short_title: "Sweep period".to_string(),
            unit_name: "".to_string(),
            step_count: SWEEP_PERIOD.max - SWEEP_PERIOD.min - 1,
            default_value: 0.0,
        },
    );

    static STUTTER_TIME: NonLinearParameter = NonLinearParameter {
        plain_zero: 0.001,
        plain_min: 0.002,
        plain_max: 1.0,
        plain_one: 1.0,
        factor: 2.0,
        diverge: true,
    };
    params.insert(
        SoyBoyParameter::StutterTime,
        ParameterDef {
            r#type: ParameterType::NonLinear,
            parameter: ParameterInfo {
                non_linear: STUTTER_TIME,
            },
            title: "Stutter time".to_string(),
            short_title: "Stutter time".to_string(),
            unit_name: "s".to_string(),
            step_count: 0,
            default_value: 0.05,
        },
    );
    static STUTTER_DEPTH: LinearParameter = LinearParameter {
        min: 0.0,
        max: 100.0,
    };
    params.insert(
        SoyBoyParameter::StutterDepth,
        ParameterDef {
            r#type: ParameterType::Linear,
            parameter: ParameterInfo {
                linear: STUTTER_DEPTH,
            },
            title: "Stutter Depth".to_string(),
            short_title: "Stutter Depth".to_string(),
            unit_name: "%".to_string(),
            step_count: 0,
            default_value: 0.0,
        },
    );
}

pub fn make_square_oscillator_parameters(params: &mut HashMap<SoyBoyParameter, ParameterDef>) {
    static SQUARE_OSCILLATOR_DUTY_LIST: [&str; 4] = ["12.5%", "25%", "50%", "75%"];
    static OSC_SQ_DUTY: ListParameter = ListParameter {
        elements: &SQUARE_OSCILLATOR_DUTY_LIST,
    };
    params.insert(
        SoyBoyParameter::OscSqDuty,
        ParameterDef {
            r#type: ParameterType::List,
            parameter: ParameterInfo { list: OSC_SQ_DUTY },
            title: "OscSq: Duty".to_string(),
            short_title: "Duty".to_string(),
            unit_name: "".to_string(),
            step_count: (OSC_SQ_DUTY.denormalize(1.0)) as i32,
            default_value: 2.0,
        },
    );
}

pub fn make_noise_oscillator_parameters(params: &mut HashMap<SoyBoyParameter, ParameterDef>) {
    static OSC_NS_INTERVAL: NonLinearParameter = NonLinearParameter {
        plain_zero: 0.001,
        plain_min: 0.002,
        plain_max: 1.0,
        plain_one: 1.0,
        factor: 2.0,
        diverge: true,
    };
    params.insert(
        SoyBoyParameter::OscNsInterval,
        ParameterDef {
            r#type: ParameterType::NonLinear,
            parameter: ParameterInfo {
                non_linear: OSC_NS_INTERVAL,
            },
            title: "OscNs: Noise interval".to_string(),
            short_title: "Noise int".to_string(),
            unit_name: "ms".to_string(),
            step_count: 0,
            default_value: 0.05,
        },
    );
}

pub fn make_wavetable_oscillator_parameters(_params: &mut HashMap<SoyBoyParameter, ParameterDef>) {}

pub fn make_envelope_generator_parameters(params: &mut HashMap<SoyBoyParameter, ParameterDef>) {
    static EG_TIME: NonLinearParameter = NonLinearParameter {
        plain_zero: 0.00,
        plain_min: 0.01,
        plain_max: 2.0,
        plain_one: 2.0,
        factor: 1.4,
        diverge: true,
    };
    params.insert(
        SoyBoyParameter::EgAttack,
        ParameterDef {
            r#type: ParameterType::NonLinear,
            parameter: ParameterInfo {
                non_linear: EG_TIME,
            },
            title: "Eg: Attack".to_string(),
            short_title: "Attack".to_string(),
            unit_name: "s".to_string(),
            step_count: 0,
            default_value: 0.08,
        },
    );
    params.insert(
        SoyBoyParameter::EgDecay,
        ParameterDef {
            r#type: ParameterType::NonLinear,
            parameter: ParameterInfo {
                non_linear: EG_TIME,
            },
            title: "Eg: Decay".to_string(),
            short_title: "Decay".to_string(),
            unit_name: "s".to_string(),
            step_count: 0,
            default_value: 0.1,
        },
    );
    params.insert(
        SoyBoyParameter::EgRelease,
        ParameterDef {
            r#type: ParameterType::NonLinear,
            parameter: ParameterInfo {
                non_linear: EG_TIME,
            },
            title: "Eg: Release".to_string(),
            short_title: "Release".to_string(),
            unit_name: "s".to_string(),
            step_count: 0,
            default_value: 0.1,
        },
    );
    static EG_SUSTAIN: LinearParameter = LinearParameter { min: 0.0, max: 1.0 };
    params.insert(
        SoyBoyParameter::EgSustain,
        ParameterDef {
            r#type: ParameterType::Linear,
            parameter: ParameterInfo { linear: EG_SUSTAIN },
            title: "Eg: Sustain".to_string(),
            short_title: "Sustain".to_string(),
            unit_name: "".to_string(),
            step_count: 0,
            default_value: 0.3,
        },
    );
}

fn make_dac_parameters(params: &mut HashMap<SoyBoyParameter, ParameterDef>) {
    static DAC_FREQ: NonLinearParameter = NonLinearParameter {
        plain_zero: 40.00,
        plain_min: 40.0,
        plain_max: 22_000.0,
        plain_one: 22_000.0,
        factor: 1.9,
        diverge: true,
    };
    params.insert(
        SoyBoyParameter::DacFreq,
        ParameterDef {
            r#type: ParameterType::NonLinear,
            parameter: ParameterInfo {
                non_linear: DAC_FREQ,
            },
            title: "Dac: freq".to_string(),
            short_title: "freq".to_string(),
            unit_name: "Hz".to_string(),
            step_count: 0,
            default_value: 22_000.0,
        },
    );
    static DAC_Q: NonLinearParameter = NonLinearParameter {
        plain_zero: 0.005,
        plain_min: 1.0,
        plain_max: 40.0,
        plain_one: 40.0,
        factor: 2.0,
        diverge: true,
    };
    params.insert(
        SoyBoyParameter::DacQ,
        ParameterDef {
            r#type: ParameterType::NonLinear,
            parameter: ParameterInfo { non_linear: DAC_Q },
            title: "Dac: Q".to_string(),
            short_title: "Q".to_string(),
            unit_name: "".to_string(),
            step_count: 0,
            default_value: 0.005,
        },
    );
}

pub fn make_parameter_info() -> HashMap<SoyBoyParameter, ParameterDef> {
    let mut params = HashMap::new();

    make_global_parameters(&mut params);

    make_square_oscillator_parameters(&mut params);
    make_noise_oscillator_parameters(&mut params);
    make_wavetable_oscillator_parameters(&mut params);

    make_envelope_generator_parameters(&mut params);

    make_dac_parameters(&mut params);

    params
}
