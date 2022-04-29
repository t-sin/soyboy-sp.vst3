use std::fmt;

use super::waveform::Waveform;

pub enum GUIThreadMessage {
    Terminate,
}

#[derive(PartialEq, Eq)]
pub enum GUIEvent {
    Redraw,
    NoteOn,
    WaveTableData([i8; 32]),
    WaveformData(Waveform),
}

pub enum Vst3Message {
    NoteOn,
    InitializeWaveTable,
    RandomizeWaveTable,
    WaveTableRequested,
    WaveTableData([i8; 32]),
    SetWaveTable(usize, i8),
    WaveformData(Waveform),
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
        };

        write!(f, "{}", s)
    }
}
