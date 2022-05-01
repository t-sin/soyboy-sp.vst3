use serde::{Deserialize, Serialize};

use crate::soyboy::parameters::{Parametric, SoyBoyParameter};

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginConfigV01 {
    pub waveform_view_enabled: bool,
    // soyboy parameters
    pub master_volume: f64,
    pub pitch_bend: f64,
    pub detune: f64,
    pub oscillator_type: f64,
    pub sweep_type: f64,
    pub sweep_amount: f64,
    pub sweep_period: f64,
    pub stutter_time: f64,
    pub stutter_depth: f64,
    pub envelope_attack: f64,
    pub envelope_decay: f64,
    pub envelope_sustain: f64,
    pub envelope_release: f64,
    pub osc_sq_duty: f64,
    pub osc_noise_interval: f64,
    pub dac_freq: f64,
    pub dac_q: f64,
}

impl PluginConfigV01 {
    /// This version is for versioning configuration data.
    /// So this is not equal to Cargo.toml's one.
    pub const CONFIG_VERSION: u32 = 1;
}

impl Parametric<SoyBoyParameter> for PluginConfigV01 {
    fn set_param(&mut self, param: &SoyBoyParameter, value: f64) {
        match param {
            SoyBoyParameter::MasterVolume => self.master_volume = value,
            SoyBoyParameter::PitchBend => self.pitch_bend = value,
            SoyBoyParameter::Detune => self.detune = value,
            SoyBoyParameter::OscillatorType => self.oscillator_type = value,
            SoyBoyParameter::SweepType => self.sweep_type = value,
            SoyBoyParameter::SweepAmount => self.sweep_amount = value,
            SoyBoyParameter::SweepPeriod => self.sweep_period = value,
            SoyBoyParameter::StutterTime => self.stutter_time = value,
            SoyBoyParameter::StutterDepth => self.stutter_depth = value,
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
            SoyBoyParameter::SweepType => self.sweep_type,
            SoyBoyParameter::SweepAmount => self.sweep_amount,
            SoyBoyParameter::SweepPeriod => self.sweep_period,
            SoyBoyParameter::StutterTime => self.stutter_time,
            SoyBoyParameter::StutterDepth => self.stutter_depth,
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

impl Default for PluginConfigV01 {
    fn default() -> Self {
        Self {
            waveform_view_enabled: false,
            master_volume: 0.0,
            pitch_bend: 0.0,
            detune: 0.0,
            oscillator_type: 0.0,
            sweep_type: 0.0,
            sweep_amount: 0.0,
            sweep_period: 0.0,
            stutter_time: 0.0,
            stutter_depth: 0.0,
            envelope_attack: 0.0,
            envelope_decay: 0.0,
            envelope_sustain: 0.0,
            envelope_release: 0.0,
            osc_sq_duty: 0.0,
            osc_noise_interval: 0.0,
            dac_freq: 0.0,
            dac_q: 0.0,
        }
    }
}
