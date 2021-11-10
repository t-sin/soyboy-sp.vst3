use std::collections::HashMap;

use crate::soyboy::Parameter;
use crate::vst3::util;

#[derive(Clone, Copy, Debug)]
pub enum ParameterType {
    Decibel,
}

pub trait Normalizable<T> {
    fn denormalize(&self, normalized: f64) -> T;
    fn normalize(&self, plain: T) -> f64;
    fn raw_denormalize(&self, normalized: f64) -> T;
    fn raw_normalize(&self, plain: T) -> f64;
}

#[derive(Clone, Copy, Debug)]
pub struct DecibelParameter {
    plain_zero: f64,
    plain_min: f64,
    plain_max: f64,
    plain_one: f64,
    exponential: f64,
    negate: bool,
}

impl Normalizable<f64> for DecibelParameter {
    fn denormalize(&self, normalized: f64) -> f64 {
        util::exponential_denormalize(
            normalized,
            self.plain_zero,
            self.plain_one,
            self.plain_min,
            self.plain_max,
            self.exponential,
            self.negate,
        )
    }

    fn normalize(&self, plain: f64) -> f64 {
        util::exponential_normalize(
            plain,
            self.plain_zero,
            self.plain_one,
            self.plain_min,
            self.plain_max,
            self.exponential,
            self.negate,
        )
    }

    fn raw_denormalize(&self, normalized: f64) -> f64 {
        util::denormalize(
            normalized,
            self.plain_zero,
            self.plain_one,
            self.plain_min,
            self.plain_max,
        )
    }

    fn raw_normalize(&self, plain: f64) -> f64 {
        util::normalize(
            plain,
            self.plain_zero,
            self.plain_one,
            self.plain_min,
            self.plain_max,
        )
    }
}

#[derive(Copy, Clone)]
pub union ParameterInfo {
    pub decibel: DecibelParameter,
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
            ParameterType::Decibel => unsafe { self.parameter.decibel.denormalize(normalized) },
        }
    }

    fn normalize(&self, plain: f64) -> f64 {
        match self.r#type {
            ParameterType::Decibel => unsafe { self.parameter.decibel.normalize(plain) },
        }
    }

    fn raw_denormalize(&self, normalized: f64) -> f64 {
        match self.r#type {
            ParameterType::Decibel => unsafe { self.parameter.decibel.denormalize(normalized) },
        }
    }

    fn raw_normalize(&self, plain: f64) -> f64 {
        match self.r#type {
            ParameterType::Decibel => unsafe { self.parameter.decibel.normalize(plain) },
        }
    }
}

pub fn make_parameter_info() -> HashMap<Parameter, SoyBoyParameter> {
    let mut params = HashMap::new();

    let param = DecibelParameter {
        plain_zero: -f64::INFINITY,
        plain_min: -110.0,
        plain_max: 6.0,
        plain_one: 6.0,
        exponential: 3.0,
        negate: true,
    };

    println!("--- init ---");
    let v = 1.0;
    println!(
        "v = {}, norm = {}, denorm = {}, denorm(norm(v)) = {}",
        v,
        param.normalize(v),
        param.denormalize(v),
        param.denormalize(param.normalize(v)),
    );
    println!("------------");
    params.insert(
        Parameter::MasterVolume,
        SoyBoyParameter {
            r#type: ParameterType::Decibel,
            parameter: ParameterInfo { decibel: param },
            title: "Master Volume".to_string(),
            short_title: "Volume".to_string(),
            unit_name: "dB".to_string(),
            step_count: 0,
            default_value: 1.0,
        },
    );

    params
}
