use log::*;

use std::cell::RefCell;
use std::os::raw::c_void;
use std::ptr::null_mut;

use vst3_com::{sys::GUID, IID};
use vst3_sys::{
    base::{
        kInvalidArgument, kResultFalse, kResultOk, kResultTrue, tresult, FIDString, IPluginBase,
        TBool,
    },
    vst::{
        AudioBusBuffers, BusDirections, BusFlags, BusInfo, BusTypes, EventTypes, IAudioProcessor,
        IComponent, IEditController, IEventList, MediaTypes, ParameterInfo, ProcessData,
        ProcessSetup, RoutingInfo, TChar, K_SAMPLE32, K_SAMPLE64,
    },
    VST3,
};

use crate::constant;
use crate::vst3::util;

use crate::gbi::{AudioProcessor, GameBoyInstrument};

#[VST3(implements(IComponent, IAudioProcessor, IEditController))]
pub struct GameBoyPlugin {
    gbi: RefCell<GameBoyInstrument>,
}

impl GameBoyPlugin {
    pub const CID: GUID = GUID {
        data: constant::VST3_CID,
    };

    pub fn new(gbi: GameBoyInstrument) -> Box<Self> {
        let gbi = RefCell::new(gbi);
        let gb = GameBoyPlugin::allocate(gbi);
        gb
    }

    pub fn bus_count(&self, media_type: MediaTypes, dir: BusDirections) -> i32 {
        match media_type {
            MediaTypes::kAudio => match dir {
                BusDirections::kInput => 0,
                BusDirections::kOutput => 1,
            },
            MediaTypes::kEvent => match dir {
                BusDirections::kInput => 1,
                BusDirections::kOutput => 0,
            },
            _ => 0,
        }
    }

    pub fn bus_info(
        &self,
        media_type: MediaTypes,
        dir: BusDirections,
        idx: i32,
    ) -> Option<(&str, BusInfo)> {
        match media_type {
            MediaTypes::kAudio => match dir {
                BusDirections::kInput => None,
                BusDirections::kOutput => match idx {
                    0 => Some((
                        "Audio Out",
                        BusInfo {
                            media_type: media_type as i32,
                            direction: dir as i32,
                            channel_count: 2, // stereo out
                            name: [0; 128],
                            bus_type: BusTypes::kMain as i32,
                            flags: BusFlags::kDefaultActive as u32,
                        },
                    )),
                    _ => None,
                },
            },
            MediaTypes::kEvent => match dir {
                BusDirections::kInput => match idx {
                    0 => Some((
                        "Event In",
                        BusInfo {
                            media_type: media_type as i32,
                            direction: dir as i32,
                            channel_count: 1,
                            name: [0; 128],
                            bus_type: BusTypes::kMain as i32,
                            flags: BusFlags::kDefaultActive as u32,
                        },
                    )),
                    _ => None,
                },
                BusDirections::kOutput => None,
            },
            _ => None,
        }
    }
}

impl IPluginBase for GameBoyPlugin {
    unsafe fn initialize(&self, _host_context: *mut c_void) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}

impl IComponent for GameBoyPlugin {
    unsafe fn get_controller_class_id(&self, _tuid: *mut IID) -> tresult {
        kResultOk
    }

    unsafe fn set_io_mode(&self, _mode: i32) -> tresult {
        kResultOk
    }

    unsafe fn get_bus_count(&self, media_type: i32, dir: i32) -> i32 {
        if let Some(media_type) = util::as_media_type(media_type) {
            if let Some(dir) = util::as_bus_dir(dir) {
                return self.bus_count(media_type, dir);
            }
        }
        0
    }

    unsafe fn get_bus_info(
        &self,
        media_type: i32,
        dir: i32,
        idx: i32,
        info: *mut BusInfo,
    ) -> tresult {
        let info = &mut *info;

        if let Some(media_type) = util::as_media_type(media_type) {
            if let Some(dir) = util::as_bus_dir(dir) {
                if let Some((name, bus_info)) = self.bus_info(media_type, dir, idx) {
                    info.direction = bus_info.direction as i32;
                    info.bus_type = bus_info.bus_type as i32;
                    info.channel_count = bus_info.channel_count;
                    info.flags = bus_info.flags as u32;
                    util::wstrcpy(name, info.name.as_mut_ptr());

                    return kResultOk;
                }
            }
        }

        kInvalidArgument
    }

    unsafe fn get_routing_info(
        &self,
        _in_info: *mut RoutingInfo,
        _out_info: *mut RoutingInfo,
    ) -> i32 {
        kResultFalse
    }

    unsafe fn activate_bus(&self, _type_: i32, _dir: i32, _idx: i32, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn set_active(&self, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn set_state(&self, _state: *mut c_void) -> tresult {
        kResultOk
    }

    unsafe fn get_state(&self, _state: *mut c_void) -> tresult {
        kResultOk
    }
}

impl IAudioProcessor for GameBoyPlugin {
    unsafe fn set_bus_arrangements(
        &self,
        _inputs: *mut u64,
        _num_inputs: i32,
        _outputs: *mut u64,
        _num_outputs: i32,
    ) -> i32 {
        kResultOk
    }

    unsafe fn get_bus_arrangement(&self, _dir: i32, _idx: i32, arr: *mut u64) -> i32 {
        let arr = &mut *arr;
        if (*arr == 0x0) || (*arr == 0x1) || (*arr == 0x3) {
            kResultOk
        } else {
            *arr = 0x03;
            kResultOk
        }
    }

    unsafe fn can_process_sample_size(&self, symbolic_sample_size: i32) -> i32 {
        match symbolic_sample_size {
            K_SAMPLE32 => kResultTrue,
            K_SAMPLE64 => kResultTrue,
            _ => kResultFalse,
        }
    }

    unsafe fn get_latency_samples(&self) -> u32 {
        0
    }

    unsafe fn setup_processing(&self, _setup: *const ProcessSetup) -> tresult {
        kResultOk
    }

    unsafe fn set_processing(&self, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn get_tail_samples(&self) -> u32 {
        0
    }

    unsafe fn process(&self, data: *mut ProcessData) -> tresult {
        let data = &*data;

        if data.input_events.is_null() || data.outputs.is_null() {
            return kResultOk;
        }

        // process event inputs
        if !data.input_events.is_null() {
            let input_events = data.input_events.upgrade().unwrap();
            let count = input_events.get_event_count();

            for c in 0..count {
                let mut e = util::make_empty_event();

                if input_events.get_event(c, &mut e) == kResultOk {
                    let mut gbi = self.gbi.borrow_mut();
                    match util::as_event_type(e.type_) {
                        Some(EventTypes::kNoteOnEvent) => gbi.note_on(e.event.note_on.pitch),
                        Some(EventTypes::kNoteOffEvent) => gbi.note_off(),
                        Some(_) => (),
                        _ => (),
                    }
                }
            }
        }

        // process audio outputs
        let num_samples = data.num_samples as usize;
        let outputs: &mut AudioBusBuffers = &mut *data.outputs;
        let num_output_channels = outputs.num_channels as usize;

        let sample_rate = (*(data.context)).sample_rate;

        let out = (*(*data).outputs).buffers;

        match data.symbolic_sample_size {
            K_SAMPLE32 => {
                for n in 0..num_samples as isize {
                    let s = self.gbi.borrow_mut().process(sample_rate);

                    for i in 0..num_output_channels as isize {
                        let ch_out = *out.offset(i) as *mut f32;
                        *ch_out.offset(n) = s.0 as f32;
                    }
                }

                kResultOk
            }
            K_SAMPLE64 => {
                for n in 0..num_samples as isize {
                    let s = self.gbi.borrow_mut().process(sample_rate);

                    for i in 0..num_output_channels as isize {
                        let ch_out = *out.offset(i) as *mut f64;
                        *ch_out.offset(n) = s.0;
                    }
                }

                kResultOk
            }
            _ => unreachable!(),
        }
    }
}
