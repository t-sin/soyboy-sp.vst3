use std::fmt;

use super::config::PluginConfigV01;
use super::waveform::Waveform;
use crate::common::{constants, i4};
use crate::soyboy::parameters::SoyBoyParameter;

pub enum GUIThreadMessage {
    Terminate,
}

#[derive(PartialEq)]
pub enum GUIEvent {
    NoteOn,
    WaveTableData([i4; constants::WAVETABLE_SIZE]),
    WaveformData(Waveform),
    Configure(PluginConfigV01),
    SetParam(SoyBoyParameter, f64),
}

pub enum Vst3Message {
    NoteOn,
    InitializeWaveTable,
    RandomizeWaveTable,
    WaveTableRequested,
    WaveTableData([i4; constants::WAVETABLE_SIZE]),
    SetWaveTable(usize, i4),
    WaveformData(Waveform),
    EnableWaveform,
    DisableWaveform,
}

impl fmt::Display for Vst3Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Vst3Message::NoteOn => "vst3:note-on",
            Vst3Message::InitializeWaveTable => "vst3:initialize-wavetable",
            Vst3Message::RandomizeWaveTable => "vst3:randomize-wavetable",
            Vst3Message::WaveTableData(_) => "vst3:wavetable-data",
            Vst3Message::WaveTableRequested => "vst3:wavetable-requested",
            Vst3Message::SetWaveTable(_, _) => "vst3:set-wavetable-sample",
            Vst3Message::WaveformData(_) => "vst3:waveform-data",
            Vst3Message::EnableWaveform => "vst3:enable-waveform",
            Vst3Message::DisableWaveform => "vst3:disable-waveform",
        };

        write!(f, "{}", s)
    }
}
