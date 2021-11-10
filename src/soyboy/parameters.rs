use std::convert::TryFrom;

#[derive(Copy, Clone)]
pub enum Parameter {
    MasterVolume = 0,
    AttackTime,
    DecayTime,
    Sustain,
    ReleaseTime,
}

impl TryFrom<u32> for Parameter {
    type Error = ();

    fn try_from(id: u32) -> Result<Self, Self::Error> {
        if id == Parameter::MasterVolume as u32 {
            Ok(Parameter::MasterVolume)
        } else if id == Parameter::AttackTime as u32 {
            Ok(Parameter::AttackTime)
        } else if id == Parameter::DecayTime as u32 {
            Ok(Parameter::DecayTime)
        } else if id == Parameter::Sustain as u32 {
            Ok(Parameter::Sustain)
        } else if id == Parameter::ReleaseTime as u32 {
            Ok(Parameter::ReleaseTime)
        } else {
            Err(())
        }
    }
}

pub trait Parametric<Parameter> {
    fn set_param(&mut self, param: &Parameter, value: f64);
    fn get_param(&self, param: &Parameter) -> f64;
}
