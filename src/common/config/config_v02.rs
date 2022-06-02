use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::common::{constants, i4};
use crate::soyboy::parameters::{ParameterDef, Parametric, SoyBoyParameter};

use super::PluginConfigV01;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginConfigV02 {
    pub waveform_view_enabled: bool,
    // soyboy parameters
    pub master_volume: f64,
    pub pitch_bend: f64,
    pub detune: f64,
    pub oscillator_type: f64,
    pub num_voices: f64,
    pub sweep_type: f64,
    pub sweep_amount: f64,
    pub sweep_period: f64,
    pub stutter_time: f64,
    pub stutter_depth: f64,
    pub stutter_when: f64,
    pub envelope_attack: f64,
    pub envelope_decay: f64,
    pub envelope_sustain: f64,
    pub envelope_release: f64,
    pub osc_sq_duty: f64,
    pub osc_noise_interval: f64,
    pub dac_freq: f64,
    pub dac_q: f64,
    pub wavetable: [i4; constants::WAVETABLE_SIZE],
}

impl PluginConfigV02 {
    /// This version is for versioning configuration data.
    /// So this is not equal to Cargo.toml's one.
    pub const CONFIG_VERSION: u32 = 2;

    pub fn set_wavetable_sample(&mut self, idx: usize, v: i4) {
        self.wavetable[idx] = v;
    }

    pub fn set_wavetable(&mut self, wavetable: &[i4; constants::WAVETABLE_SIZE]) {
        self.wavetable = wavetable.clone();
    }

    pub fn from_v01(
        v01: PluginConfigV01,
        param_defs: &HashMap<SoyBoyParameter, ParameterDef>,
    ) -> Self {
        let mut v02 = Self::default();

        for param in SoyBoyParameter::iter() {
            let param_def = param_defs.get(&param).unwrap();
            let v = if let SoyBoyParameter::StutterWhen = param {
                param_def.default_value
            } else {
                v01.get_param(&param)
            };

            v02.set_param(&param, param_def, v);
        }
        v02.set_wavetable(&v01.wavetable);

        v02
    }
}

impl Parametric<SoyBoyParameter> for PluginConfigV02 {
    fn set_param(&mut self, param: &SoyBoyParameter, param_def: &ParameterDef, value: f64) {
        let value = param_def.clamp(value);

        match param {
            SoyBoyParameter::MasterVolume => self.master_volume = value,
            SoyBoyParameter::PitchBend => self.pitch_bend = value,
            SoyBoyParameter::Detune => self.detune = value,
            SoyBoyParameter::OscillatorType => self.oscillator_type = value,
            SoyBoyParameter::NumVoices => self.num_voices = value,
            SoyBoyParameter::SweepType => self.sweep_type = value,
            SoyBoyParameter::SweepAmount => self.sweep_amount = value,
            SoyBoyParameter::SweepPeriod => self.sweep_period = value,
            SoyBoyParameter::StutterTime => self.stutter_time = value,
            SoyBoyParameter::StutterDepth => self.stutter_depth = value,
            SoyBoyParameter::StutterWhen => self.stutter_when = value,
            SoyBoyParameter::EgAttack => self.envelope_attack = value,
            SoyBoyParameter::EgDecay => self.envelope_decay = value,
            SoyBoyParameter::EgSustain => self.envelope_sustain = value,
            SoyBoyParameter::EgRelease => self.envelope_release = value,
            SoyBoyParameter::OscSqDuty => self.osc_sq_duty = value,
            SoyBoyParameter::OscNsInterval => self.osc_noise_interval = value,
            SoyBoyParameter::DacFreq => self.dac_freq = value,
            SoyBoyParameter::DacQ => self.dac_q = value,
        }
    }

    fn get_param(&self, param: &SoyBoyParameter) -> f64 {
        match param {
            SoyBoyParameter::MasterVolume => self.master_volume,
            SoyBoyParameter::PitchBend => self.pitch_bend,
            SoyBoyParameter::Detune => self.detune,
            SoyBoyParameter::OscillatorType => self.oscillator_type,
            SoyBoyParameter::NumVoices => self.num_voices,
            SoyBoyParameter::SweepType => self.sweep_type,
            SoyBoyParameter::SweepAmount => self.sweep_amount,
            SoyBoyParameter::SweepPeriod => self.sweep_period,
            SoyBoyParameter::StutterTime => self.stutter_time,
            SoyBoyParameter::StutterDepth => self.stutter_depth,
            SoyBoyParameter::StutterWhen => self.stutter_when,
            SoyBoyParameter::EgAttack => self.envelope_attack,
            SoyBoyParameter::EgDecay => self.envelope_decay,
            SoyBoyParameter::EgSustain => self.envelope_sustain,
            SoyBoyParameter::EgRelease => self.envelope_release,
            SoyBoyParameter::OscSqDuty => self.osc_sq_duty,
            SoyBoyParameter::OscNsInterval => self.osc_noise_interval,
            SoyBoyParameter::DacFreq => self.dac_freq,
            SoyBoyParameter::DacQ => self.dac_q,
        }
    }
}

impl Default for PluginConfigV02 {
    fn default() -> Self {
        Self {
            waveform_view_enabled: false,
            master_volume: 0.0,
            pitch_bend: 0.0,
            detune: 0.0,
            oscillator_type: 0.0,
            num_voices: 0.0,
            sweep_type: 0.0,
            sweep_amount: 0.0,
            sweep_period: 0.0,
            stutter_time: 0.0,
            stutter_depth: 0.0,
            stutter_when: 0.0,
            envelope_attack: 0.0,
            envelope_decay: 0.0,
            envelope_sustain: 0.0,
            envelope_release: 0.0,
            osc_sq_duty: 0.0,
            osc_noise_interval: 0.0,
            dac_freq: 0.0,
            dac_q: 0.0,
            wavetable: [i4::from(0i8); constants::WAVETABLE_SIZE],
        }
    }
}

impl PartialEq for PluginConfigV02 {
    fn eq(&self, _other: &Self) -> bool {
        false
    }

    fn ne(&self, _other: &Self) -> bool {
        true
    }
}
impl Eq for PluginConfigV02 {}
