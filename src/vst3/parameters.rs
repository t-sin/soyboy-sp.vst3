use std::collections::HashMap;

use crate::soyboy::Parameter;
use crate::vst3::util;

#[derive(Clone, Copy, Debug)]
pub enum ParameterType {
    NonLinear,
    Linear,
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
                util::divergent_denormalize
            } else {
                util::convergent_denormalize
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
                util::divergent_normalize
            } else {
                util::convergent_normalize
            };
            normalizer(plain, self.plain_min, self.plain_max, self.factor)
        }
    }

    fn format(&self, normalized: f64) -> String {
        format!("{:.2}", self.denormalize(normalized))
    }

    fn parse(&self, string: &str) -> Option<f64> {
        if let Some(vs) = string.split(' ').nth(0) {
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
        util::linear_denormalize(normalized, self.min, self.max)
    }

    fn normalize(&self, plain: f64) -> f64 {
        util::linear_normalize(plain, self.min, self.max)
    }

    fn format(&self, normalized: f64) -> String {
        format!("{:.2}", self.denormalize(normalized))
    }

    fn parse(&self, string: &str) -> Option<f64> {
        if let Some(vs) = string.split(' ').nth(0) {
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
pub union ParameterInfo {
    pub non_linear: NonLinearParameter,
    pub linear: LinearParameter,
}

#[derive(Clone)]
pub struct SoyBoyParameter {
    pub r#type: ParameterType,
    pub parameter: ParameterInfo,
    pub title: String,
    pub short_title: String,
    pub unit_name: String,
    pub step_count: i32,
    pub default_value: f64,
}

impl Normalizable<f64> for SoyBoyParameter {
    fn denormalize(&self, normalized: f64) -> f64 {
        match self.r#type {
            ParameterType::NonLinear => unsafe {
                self.parameter.non_linear.denormalize(normalized)
            },
            ParameterType::Linear => unsafe { self.parameter.linear.denormalize(normalized) },
        }
    }

    fn normalize(&self, plain: f64) -> f64 {
        match self.r#type {
            ParameterType::NonLinear => unsafe { self.parameter.non_linear.normalize(plain) },
            ParameterType::Linear => unsafe { self.parameter.linear.normalize(plain) },
        }
    }

    fn format(&self, normalized: f64) -> String {
        let s = match self.r#type {
            ParameterType::NonLinear => unsafe { self.parameter.non_linear.format(normalized) },
            ParameterType::Linear => unsafe { self.parameter.linear.format(normalized) },
        };
        format!("{} {}", s, self.unit_name)
    }

    fn parse(&self, string: &str) -> Option<f64> {
        match self.r#type {
            ParameterType::NonLinear => unsafe { self.parameter.non_linear.parse(string) },
            ParameterType::Linear => unsafe { self.parameter.linear.parse(string) },
        }
    }
}

pub fn make_parameter_info() -> HashMap<Parameter, SoyBoyParameter> {
    let mut params = HashMap::new();

    // global parameters
    let param = NonLinearParameter {
        plain_zero: -f64::INFINITY,
        plain_min: -110.0,
        plain_max: 6.0,
        plain_one: 6.0,
        factor: 10.0,
        diverge: false,
    };
    params.insert(
        Parameter::MasterVolume,
        SoyBoyParameter {
            r#type: ParameterType::NonLinear,
            parameter: ParameterInfo { non_linear: param },
            title: "Master Volume".to_string(),
            short_title: "Volume".to_string(),
            unit_name: "dB".to_string(),
            step_count: 0,
            default_value: param.normalize(1.0),
        },
    );

    // envelope generator parameters
    let param = NonLinearParameter {
        plain_zero: 0.00,
        plain_min: 0.01,
        plain_max: 2.0,
        plain_one: 2.0,
        factor: 1.4,
        diverge: true,
    };
    params.insert(
        Parameter::AttackTime,
        SoyBoyParameter {
            r#type: ParameterType::NonLinear,
            parameter: ParameterInfo {
                non_linear: param.clone(),
            },
            title: "EG: Attack".to_string(),
            short_title: "Attack".to_string(),
            unit_name: "s".to_string(),
            step_count: 0,
            default_value: param.normalize(0.08),
        },
    );
    params.insert(
        Parameter::DecayTime,
        SoyBoyParameter {
            r#type: ParameterType::NonLinear,
            parameter: ParameterInfo {
                non_linear: param.clone(),
            },
            title: "EG: Decay".to_string(),
            short_title: "Decay".to_string(),
            unit_name: "s".to_string(),
            step_count: 0,
            default_value: param.normalize(0.1),
        },
    );
    params.insert(
        Parameter::ReleaseTime,
        SoyBoyParameter {
            r#type: ParameterType::NonLinear,
            parameter: ParameterInfo {
                non_linear: param.clone(),
            },
            title: "EG: Release".to_string(),
            short_title: "Release".to_string(),
            unit_name: "s".to_string(),
            step_count: 0,
            default_value: param.normalize(0.1),
        },
    );
    let param = LinearParameter { min: 0.0, max: 1.0 };
    params.insert(
        Parameter::Sustain,
        SoyBoyParameter {
            r#type: ParameterType::Linear,
            parameter: ParameterInfo { linear: param },
            title: "EG: Sustain".to_string(),
            short_title: "Sustain".to_string(),
            unit_name: "".to_string(),
            step_count: 0,
            default_value: param.normalize(0.3),
        },
    );

    params
}
