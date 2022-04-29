use std::ffi::CString;
use std::fmt;
use std::os::raw::c_void;
use std::ptr::null_mut;

use vst3_com::ComInterface;
use vst3_sys::{
    base::kResultOk,
    utils::SharedVstPtr,
    vst::{IAttributeList, IHostApplication, IMessage},
    VstPtr,
};

use crate::common::{constants, Waveform};
use crate::vst3::{common::ComPtr, raw_utils::fidstring_to_string};

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

impl Vst3Message {
    pub fn from_message(msg: &SharedVstPtr<dyn IMessage>) -> Option<Self> {
        let msg = msg.upgrade().unwrap();
        let id = unsafe { fidstring_to_string(msg.get_message_id()) };

        match id.as_str() {
            "vst3:note-on" => Some(Vst3Message::NoteOn),
            "vst3:initialize-wavetable" => Some(Vst3Message::InitializeWaveTable),
            "vst3:randomize-wavetable" => Some(Vst3Message::RandomizeWaveTable),
            "vst3:wavetable-data" => {
                let attr = unsafe { msg.get_attributes() };
                let attr_id = CString::new("table").unwrap();
                let mut size: u32 = 0;
                let table_ptr: *mut c_void = null_mut();

                unsafe {
                    attr.upgrade().unwrap().get_binary(
                        attr_id.as_ptr(),
                        &table_ptr as *const _,
                        &mut size as *mut _,
                    );
                };

                let table_ptr = table_ptr as *mut i8;
                let table_src = unsafe { std::slice::from_raw_parts(table_ptr, size as usize) };
                let mut table: [i8; 32] = [0; 32];
                table.as_mut_slice().copy_from_slice(&table_src[..]);

                Some(Vst3Message::WaveTableData(table))
            }
            "vst3:wavetable-requested" => Some(Vst3Message::WaveTableRequested),
            "vst3:set-wavetable-sample" => {
                let attr = unsafe { msg.get_attributes() };
                let id_idx = CString::new("index").unwrap();
                let id_val = CString::new("value").unwrap();
                let mut idx: i64 = 0;
                let mut val: i64 = 0;

                unsafe {
                    attr.upgrade()
                        .unwrap()
                        .get_int(id_idx.as_ptr(), &mut idx as *mut _);
                    attr.upgrade()
                        .unwrap()
                        .get_int(id_val.as_ptr(), &mut val as *mut _);
                };

                Some(Vst3Message::SetWaveTable(idx as usize, val as i8))
            }
            "vst3:waveform-data" => {
                let attr = unsafe { msg.get_attributes() };
                let attr_id = CString::new("signals").unwrap();
                let mut size: u32 = 0;
                let signals_ptr: *mut c_void = null_mut();

                unsafe {
                    attr.upgrade().unwrap().get_binary(
                        attr_id.as_ptr(),
                        &signals_ptr as *const _,
                        &mut size as *mut _,
                    );
                };

                let signals_ptr = signals_ptr as *mut f64;
                let signals_src = unsafe {
                    std::slice::from_raw_parts(signals_ptr, constants::OSCILLOSCOPE_SAIMPLE_SIZE)
                };
                let mut signals: [f64; constants::OSCILLOSCOPE_SAIMPLE_SIZE] =
                    [0.0; constants::OSCILLOSCOPE_SAIMPLE_SIZE];
                signals
                    .as_mut_slice()
                    .copy_from_slice(&signals_src[..constants::OSCILLOSCOPE_SAIMPLE_SIZE]);

                let mut wf = Waveform::new();
                wf.set_signals(&signals);

                Some(Vst3Message::WaveformData(wf))
            }
            _ => None,
        }
    }

    fn to_cstring(&self) -> CString {
        CString::new(self.to_string()).unwrap()
    }

    fn write_message(&self, msg: &mut VstPtr<dyn IMessage>) {
        match self {
            Vst3Message::NoteOn => {
                unsafe { msg.set_message_id(self.to_cstring().as_ptr()) };
            }
            Vst3Message::InitializeWaveTable => unsafe {
                msg.set_message_id(self.to_cstring().as_ptr());
            },
            Vst3Message::RandomizeWaveTable => {
                unsafe { msg.set_message_id(self.to_cstring().as_ptr()) };
            }
            Vst3Message::WaveTableData(table) => {
                unsafe { msg.set_message_id(self.to_cstring().as_ptr()) };

                let attr = unsafe { msg.get_attributes() };
                let attr_id = CString::new("table").unwrap();
                let size = table.len() as u32;

                unsafe {
                    attr.upgrade().unwrap().set_binary(
                        attr_id.as_ptr(),
                        table.as_ptr() as *const c_void,
                        size,
                    );
                };
            }
            Vst3Message::WaveTableRequested => {
                unsafe { msg.set_message_id(self.to_cstring().as_ptr()) };
            }
            Vst3Message::SetWaveTable(idx, val) => {
                unsafe { msg.set_message_id(self.to_cstring().as_ptr()) };

                let attr = unsafe { msg.get_attributes() };
                let id_idx = CString::new("index").unwrap();
                let id_val = CString::new("value").unwrap();

                unsafe {
                    attr.upgrade()
                        .unwrap()
                        .set_int(id_idx.as_ptr(), *idx as i64);
                    attr.upgrade()
                        .unwrap()
                        .set_int(id_val.as_ptr(), *val as i64);
                };
            }
            Vst3Message::WaveformData(wf) => {
                unsafe { msg.set_message_id(self.to_cstring().as_ptr()) };

                let attr = unsafe { msg.get_attributes() };
                let attr_id = CString::new("signals").unwrap();

                let signals = wf.get_signals();
                let size = signals.len() * std::mem::size_of::<f64>();

                unsafe {
                    attr.upgrade().unwrap().set_binary(
                        attr_id.as_ptr(),
                        signals.as_ptr() as *const c_void,
                        size as u32,
                    );
                };
            }
        }
    }

    pub fn allocate(&self, host: &VstPtr<dyn IHostApplication>) -> Option<ComPtr<dyn IMessage>> {
        let iid = <dyn IMessage as ComInterface>::IID;
        let iid = &iid as *const _;
        let mut msg_ptr: *mut c_void = null_mut();

        let result = unsafe { host.create_instance(iid, iid, &mut msg_ptr as *mut _) };
        if result != kResultOk {
            #[cfg(debug_assertions)]
            print!("Vst3Message::allocate(): calling IHostApplication::create_instance() failed because ");

            return None;
        }

        let mut msg_obj = unsafe { VstPtr::shared(msg_ptr as *mut _).unwrap() };
        #[cfg(debug_assertions)]
        self.write_message(&mut msg_obj);

        Some(ComPtr::new(msg_ptr, msg_obj))
    }
}
