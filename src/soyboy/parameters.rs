use std::cmp::{Eq, PartialEq};
use std::convert::TryFrom;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Parameter {
    // global parameter
    MasterVolume = 0,
    // envelope generator
    EgAttack,
    EgDecay,
    EgSustain,
    EgRelease,
    // square wave oscilllator
    OscSqDuty,
}

impl TryFrom<u32> for Parameter {
    type Error = ();

    fn try_from(id: u32) -> Result<Self, Self::Error> {
        if id == Parameter::MasterVolume as u32 {
            Ok(Parameter::MasterVolume)
        } else if id == Parameter::EgAttack as u32 {
            Ok(Parameter::EgAttack)
        } else if id == Parameter::EgDecay as u32 {
            Ok(Parameter::EgDecay)
        } else if id == Parameter::EgSustain as u32 {
            Ok(Parameter::EgSustain)
        } else if id == Parameter::EgRelease as u32 {
            Ok(Parameter::EgRelease)
        } else if id == Parameter::OscSqDuty as u32 {
            Ok(Parameter::OscSqDuty)
        } else {
            Err(())
        }
    }
}

pub trait Parametric<Parameter> {
    fn set_param(&mut self, param: &Parameter, value: f64);
}
