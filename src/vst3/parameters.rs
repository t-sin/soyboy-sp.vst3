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

#[derive(Clone, Copy, Debug)]
pub struct NonLinearParameter {
    plain_zero: f64,
    plain_min: f64,
    plain_max: f64,
    plain_one: f64,
    factor: f64,
}

impl Normalizable<f64> for NonLinearParameter {
    fn denormalize(&self, normalized: f64) -> f64 {
        util::non_linear_denormalize(
            normalized,
            self.plain_zero,
            self.plain_one,
            self.plain_min,
            self.plain_max,
            self.factor,
        )
    }

    fn normalize(&self, plain: f64) -> f64 {
        util::non_linear_normalize(
            plain,
            self.plain_zero,
            self.plain_one,
            self.plain_min,
            self.plain_max,
            self.factor,
        )
    }

    fn format(&self, normalized: f64) -> String {
        format!("{:.2}", self.denormalize(normalized))
    }

    fn parse(&self, string: &str) -> Option<f64> {
        if let Ok(v) = string.parse() {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LinearParameter {
    min_sec: f64,
    max_sec: f64,
}

impl Normalizable<f64> for LinearParameter {
    fn denormalize(&self, normalized: f64) -> f64 {
        util::linear_denormalize(normalized, self.min_sec, self.max_sec)
    }

    fn normalize(&self, plain: f64) -> f64 {
        util::linear_normalize(plain, self.min_sec, self.max_sec)
    }

    fn format(&self, normalized: f64) -> String {
        format!("{:.2}", self.denormalize(normalized))
    }

    fn parse(&self, string: &str) -> Option<f64> {
        if let Ok(v) = string.parse() {
            Some(v)
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
        factor: 3.0,
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
        plain_zero: 0.01,
        plain_min: 0.01,
        plain_max: 5.0,
        plain_one: 5.0,
        factor: 2.0,
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
            default_value: param.normalize(0.05),
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
            default_value: param.normalize(0.05),
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
            default_value: param.normalize(0.05),
        },
    );
    let param = LinearParameter {
        min_sec: 0.0,
        max_sec: 1.0,
    };
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
