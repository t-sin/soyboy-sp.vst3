use std::convert::TryFrom;

use crate::{
    common::{constants, i4},
    soyboy::{
        dac::DAConverter,
        envelope_generator::EnvelopeGenerator,
        event::{Event, Triggered},
        noise::NoiseOscillator,
        parameters::{ParameterDef, Parametric, SoyBoyParameter},
        square_wave::SquareWaveOscillator,
        sweep::SweepOscillator,
        types::AudioProcessor,
        utils::{frequency_from_note_number, ratio_from_cents},
        wave_table::WaveTableOscillator,
    },
};

#[derive(Copy, Clone)]
enum OscillatorType {
    Square = 0,
    Noise,
    WaveTable,
}

impl TryFrom<u32> for OscillatorType {
    type Error = ();

    fn try_from(id: u32) -> Result<Self, Self::Error> {
        if id == OscillatorType::Square as u32 {
            Ok(OscillatorType::Square)
        } else if id == OscillatorType::Noise as u32 {
            Ok(OscillatorType::Noise)
        } else if id == OscillatorType::WaveTable as u32 {
            Ok(OscillatorType::WaveTable)
        } else {
            Err(())
        }
    }
}

pub struct VoiceUnit {
    note_on_freq: f64,
    freq: f64,

    square_osc: SquareWaveOscillator,
    noise_osc: NoiseOscillator,
    wavetable_osc: WaveTableOscillator,
    sweep_osc: SweepOscillator,
    dac: DAConverter,
    envelope_gen: EnvelopeGenerator,

    pitch: i16,
    detune: i16,
    selected_osc: OscillatorType,
}

impl VoiceUnit {
    pub fn new() -> Self {
        Self {
            note_on_freq: 0.0,
            freq: 0.0,

            square_osc: SquareWaveOscillator::new(),
            noise_osc: NoiseOscillator::new(),
            wavetable_osc: WaveTableOscillator::new(),
            sweep_osc: SweepOscillator::new(),
            dac: DAConverter::new(22_000.0, 0.005),
            envelope_gen: EnvelopeGenerator::new(),

            pitch: 0,
            detune: 0,
            selected_osc: OscillatorType::Square,
        }
    }

    pub fn get_wavetable(&self) -> [i4; constants::WAVETABLE_SIZE] {
        self.wavetable_osc.get_wavetable()
    }

    pub fn set_wavetable(&mut self, wavetable: &[i4; constants::WAVETABLE_SIZE]) {
        self.wavetable_osc.set_wavetable(wavetable);
    }

    pub fn same_note(&self, note: u16) -> bool {
        self.envelope_gen.same_note(note)
    }

    pub fn assignable(&self, note: u16) -> bool {
        self.envelope_gen.assignable(note)
    }
}

impl Triggered for VoiceUnit {
    fn trigger(&mut self, event: &Event) {
        match event {
            Event::NoteOn { note, velocity: _ } => {
                self.note_on_freq = frequency_from_note_number(*note);
                self.freq = self.note_on_freq;
                self.sweep_osc
                    .trigger(&Event::SweepReset { freq: self.freq });
                self.envelope_gen.trigger(event);
            }
            Event::NoteOff { note: _ } => {
                self.envelope_gen.trigger(event);
            }
            Event::PitchBend { ratio: _ } => {
                self.square_osc.trigger(event);
                self.wavetable_osc.trigger(event);
            }
            Event::SetWaveTable { .. } => self.wavetable_osc.trigger(event),
            Event::ResetWaveTableAsSine => self.wavetable_osc.trigger(event),
            Event::ResetWaveTableAtRandom => self.wavetable_osc.trigger(event),
            _ => (),
        }
    }
}

impl Parametric<SoyBoyParameter> for VoiceUnit {
    fn set_param(&mut self, param: &SoyBoyParameter, param_def: &ParameterDef, value: f64) {
        match param {
            SoyBoyParameter::PitchBend => {
                self.pitch = value as i16;
                let ratio = ratio_from_cents(self.pitch + self.detune);
                self.trigger(&Event::PitchBend { ratio });
            }
            SoyBoyParameter::Detune => {
                self.detune = value as i16;
                let ratio = ratio_from_cents(self.pitch + self.detune);
                self.trigger(&Event::PitchBend { ratio });
            }
            SoyBoyParameter::OscillatorType => {
                if let Ok(r#type) = OscillatorType::try_from(value as u32) {
                    self.selected_osc = r#type
                }
            }
            SoyBoyParameter::SweepType => {
                self.freq = self.note_on_freq;
                self.sweep_osc.set_param(param, param_def, value);
            }
            SoyBoyParameter::SweepAmount => self.sweep_osc.set_param(param, param_def, value),
            SoyBoyParameter::SweepPeriod => self.sweep_osc.set_param(param, param_def, value),
            SoyBoyParameter::StutterTime => self.envelope_gen.set_param(param, param_def, value),
            SoyBoyParameter::StutterDepth => self.envelope_gen.set_param(param, param_def, value),
            SoyBoyParameter::StutterWhen => self.envelope_gen.set_param(param, param_def, value),
            SoyBoyParameter::EgAttack => self.envelope_gen.set_param(param, param_def, value),
            SoyBoyParameter::EgDecay => self.envelope_gen.set_param(param, param_def, value),
            SoyBoyParameter::EgSustain => self.envelope_gen.set_param(param, param_def, value),
            SoyBoyParameter::EgRelease => self.envelope_gen.set_param(param, param_def, value),
            SoyBoyParameter::OscSqDuty => self.square_osc.set_param(param, param_def, value),
            SoyBoyParameter::OscNsInterval => self.noise_osc.set_param(param, param_def, value),
            SoyBoyParameter::DacFreq => self.dac.set_param(param, param_def, value),
            SoyBoyParameter::DacQ => self.dac.set_param(param, param_def, value),
            _ => (),
        }
    }

    fn get_param(&self, param: &SoyBoyParameter) -> f64 {
        match param {
            SoyBoyParameter::PitchBend => self.pitch as f64,
            SoyBoyParameter::Detune => self.detune as f64,
            SoyBoyParameter::OscillatorType => {
                let v = self.selected_osc as u32;
                v.into()
            }
            SoyBoyParameter::SweepType => self.sweep_osc.get_param(param),
            SoyBoyParameter::SweepAmount => self.sweep_osc.get_param(param),
            SoyBoyParameter::SweepPeriod => self.sweep_osc.get_param(param),
            SoyBoyParameter::StutterTime => self.envelope_gen.get_param(param),
            SoyBoyParameter::StutterDepth => self.envelope_gen.get_param(param),
            SoyBoyParameter::StutterWhen => self.envelope_gen.get_param(param),
            SoyBoyParameter::EgAttack => self.envelope_gen.get_param(param),
            SoyBoyParameter::EgDecay => self.envelope_gen.get_param(param),
            SoyBoyParameter::EgSustain => self.envelope_gen.get_param(param),
            SoyBoyParameter::EgRelease => self.envelope_gen.get_param(param),
            SoyBoyParameter::OscSqDuty => self.square_osc.get_param(param),
            SoyBoyParameter::OscNsInterval => self.noise_osc.get_param(param),
            SoyBoyParameter::DacFreq => self.dac.get_param(param),
            SoyBoyParameter::DacQ => self.dac.get_param(param),
            _ => 0.0,
        }
    }
}

impl AudioProcessor<f64> for VoiceUnit {
    fn process(&mut self, sample_rate: f64) -> f64 {
        let osc = if self.sweep_osc.is_clipped() {
            i4::ZERO.into()
        } else {
            let freq_mod = self.sweep_osc.process(sample_rate);
            self.freq += freq_mod;

            match self.selected_osc {
                OscillatorType::Square => {
                    self.square_osc.set_freq(self.freq);
                    self.square_osc.process(sample_rate)
                }
                OscillatorType::Noise => {
                    self.noise_osc.set_freq(self.freq);
                    self.noise_osc.process(sample_rate)
                }
                OscillatorType::WaveTable => {
                    self.wavetable_osc.set_freq(self.freq);
                    self.wavetable_osc.process(sample_rate)
                }
            }
        };

        let env = self.envelope_gen.process(sample_rate);

        let v = self.dac.process(sample_rate, osc * env);
        v
    }

    fn set_freq(&mut self, _freq: f64) {}
}
