use std::convert::TryFrom;

use crate::soyboy::{
    dac::DAConverter,
    envelope_generator::EnvelopeGenerator,
    event::{Event, Triggered},
    noise::NoiseOscillator,
    parameters::{Parameter, Parametric},
    square_wave::SquareWaveOscillator,
    types::AudioProcessor,
    utils::{level, ratio_from_cents},
    wave_table::WaveTableOscillator,
};

pub type Signal = (f64, f64);

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

pub struct SoyBoy {
    square_osc: SquareWaveOscillator,
    noise_osc: NoiseOscillator,
    wavetable_osc: WaveTableOscillator,
    dac: DAConverter,
    envelope_gen: EnvelopeGenerator,

    master_volume: f64,
    pitch: i16,
    detune: i16,
    selected_osc: OscillatorType,
}

impl SoyBoy {
    pub fn new() -> SoyBoy {
        SoyBoy {
            square_osc: SquareWaveOscillator::new(),
            noise_osc: NoiseOscillator::new(),
            wavetable_osc: WaveTableOscillator::new(),
            dac: DAConverter::new(22_000.0, 0.005),
            envelope_gen: EnvelopeGenerator::new(),

            master_volume: 1.0,
            pitch: 0,
            detune: 0,
            selected_osc: OscillatorType::Square,
        }
    }
}

impl Triggered for SoyBoy {
    fn trigger(&mut self, event: &Event) {
        match event {
            Event::NoteOn {
                note: _,
                velocity: _,
            } => {
                self.square_osc.trigger(event);
                self.noise_osc.trigger(event);
                self.wavetable_osc.trigger(event);

                self.envelope_gen.trigger(event);
            }
            Event::NoteOff { note: _ } => {
                self.envelope_gen.trigger(event);
            }
            Event::PitchBend { ratio: _ } => {
                self.square_osc.trigger(event);
                self.wavetable_osc.trigger(event);
            }
        }
    }
}

impl Parametric<Parameter> for SoyBoy {
    fn set_param(&mut self, param: &Parameter, value: f64) {
        match param {
            Parameter::MasterVolume => self.master_volume = value,
            Parameter::PitchBend => {
                self.pitch = value as i16;
                let ratio = ratio_from_cents(self.pitch + self.detune);
                self.trigger(&Event::PitchBend { ratio: ratio });
            }
            Parameter::Detune => {
                self.detune = value as i16;
                let ratio = ratio_from_cents(self.pitch + self.detune);
                self.trigger(&Event::PitchBend { ratio: ratio });
            }
            Parameter::OscillatorType => {
                if let Ok(r#type) = OscillatorType::try_from(value as u32) {
                    self.selected_osc = r#type
                }
            }
            Parameter::EgAttack => self.envelope_gen.set_param(param, value),
            Parameter::EgDecay => self.envelope_gen.set_param(param, value),
            Parameter::EgSustain => self.envelope_gen.set_param(param, value),
            Parameter::EgRelease => self.envelope_gen.set_param(param, value),
            Parameter::OscSqDuty => self.square_osc.set_param(param, value),
            Parameter::OscSqSweepType => self.square_osc.set_param(param, value),
            Parameter::OscSqSweepAmount => self.square_osc.set_param(param, value),
            Parameter::OscSqSweepPeriod => self.square_osc.set_param(param, value),
            Parameter::OscNsInterval => self.noise_osc.set_param(param, value),
            Parameter::OscWtTableIndex => self.wavetable_osc.set_param(param, value),
            Parameter::OscWtTableValue => self.wavetable_osc.set_param(param, value),
        }
    }

    fn get_param(&self, param: &Parameter) -> f64 {
        match param {
            Parameter::MasterVolume => self.master_volume,
            Parameter::PitchBend => self.pitch as f64,
            Parameter::Detune => self.detune as f64,
            Parameter::OscillatorType => {
                let v = self.selected_osc as u32;
                v.into()
            }
            Parameter::EgAttack => self.envelope_gen.get_param(param),
            Parameter::EgDecay => self.envelope_gen.get_param(param),
            Parameter::EgSustain => self.envelope_gen.get_param(param),
            Parameter::EgRelease => self.envelope_gen.get_param(param),
            Parameter::OscSqDuty => self.square_osc.get_param(param),
            Parameter::OscSqSweepType => self.square_osc.get_param(param),
            Parameter::OscSqSweepAmount => self.square_osc.get_param(param),
            Parameter::OscSqSweepPeriod => self.square_osc.get_param(param),
            Parameter::OscNsInterval => self.noise_osc.get_param(param),
            Parameter::OscWtTableIndex => self.wavetable_osc.get_param(param),
            Parameter::OscWtTableValue => self.wavetable_osc.get_param(param),
        }
    }
}

impl AudioProcessor<Signal> for SoyBoy {
    fn process(&mut self, sample_rate: f64) -> Signal {
        let osc = match self.selected_osc {
            OscillatorType::Square => self.square_osc.process(sample_rate),
            OscillatorType::Noise => self.noise_osc.process(sample_rate),
            OscillatorType::WaveTable => self.wavetable_osc.process(sample_rate),
        };
        let env = self.envelope_gen.process(sample_rate);

        let v = self.dac.process(sample_rate, osc * env) * level(self.master_volume);
        (v, v)
    }
}
