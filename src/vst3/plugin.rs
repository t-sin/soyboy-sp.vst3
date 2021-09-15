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
        AudioBusBuffers, BusDirections, BusFlags, BusInfo, DataEvent, Event, EventData, EventTypes,
        IAudioProcessor, IComponent, IEditController, IEventList, MediaTypes, ParameterInfo,
        ProcessData, ProcessSetup, RoutingInfo, TChar, K_SAMPLE32, K_SAMPLE64,
    },
    VST3,
};

use crate::constant;
use crate::vst3::util::wstrcpy;

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
}

impl IPluginBase for GameBoyPlugin {
    unsafe fn initialize(&self, _host_context: *mut c_void) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}

const K_AUDIO: i32 = MediaTypes::kAudio as i32;
const K_EVENT: i32 = MediaTypes::kEvent as i32;

impl IComponent for GameBoyPlugin {
    unsafe fn get_controller_class_id(&self, _tuid: *mut IID) -> tresult {
        kResultOk
    }

    unsafe fn set_io_mode(&self, _mode: i32) -> tresult {
        kResultOk
    }

    unsafe fn get_bus_count(&self, type_: i32, dir: i32) -> i32 {
        if type_ == MediaTypes::kAudio as i32 {
            if dir == BusDirections::kOutput as i32 {
                1
            } else {
                0
            }
        } else if type_ == MediaTypes::kEvent as i32 {
            if dir == BusDirections::kInput as i32 {
                1
            } else {
                0
            }
        } else {
            0
        }
    }

    unsafe fn get_bus_info(&self, type_: i32, dir: i32, _idx: i32, info: *mut BusInfo) -> tresult {
        let info = &mut *info;

        match type_ {
            K_AUDIO => {
                if dir == BusDirections::kInput as i32 {
                    kInvalidArgument
                } else if dir == BusDirections::kOutput as i32 {
                    info.direction = dir;
                    info.bus_type = MediaTypes::kAudio as i32;
                    info.channel_count = 2;
                    info.flags = BusFlags::kDefaultActive as u32;
                    wstrcpy("Audio Output", info.name.as_mut_ptr());

                    kResultOk
                } else {
                    kInvalidArgument
                }
            }
            K_EVENT => {
                if dir == BusDirections::kInput as i32 {
                    info.direction = dir;
                    info.bus_type = MediaTypes::kEvent as i32;
                    info.channel_count = 1;
                    info.flags = BusFlags::kDefaultActive as u32;
                    wstrcpy("Event Input", info.name.as_mut_ptr());

                    kResultOk
                } else {
                    kInvalidArgument
                }
            }
            _ => kInvalidArgument,
        }
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

const K_NOTE_ON_EVENT: u16 = EventTypes::kNoteOnEvent as u16;
const K_NOTE_OFF_EVENT: u16 = EventTypes::kNoteOffEvent as u16;

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
                let bytes = [0];
                let mut event: Event = Event {
                    bus_index: 0,
                    sample_offset: 0,
                    ppq_position: 0.0,
                    flags: 0,
                    type_: 0,
                    event: EventData {
                        data: DataEvent {
                            size: 0,
                            type_: 0,
                            bytes: bytes.as_ptr(),
                        },
                    },
                };

                if input_events.get_event(c, &mut event) == kResultOk {
                    println!("process_event");
                    let mut gbi = self.gbi.borrow_mut();
                    match event.type_ {
                        K_NOTE_ON_EVENT => gbi.note_on(),
                        K_NOTE_OFF_EVENT => gbi.note_off(),
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

impl IEditController for GameBoyPlugin {
    unsafe fn set_component_state(&self, _state: *mut c_void) -> tresult {
        info!("set_component_state");
        kResultOk
    }

    unsafe fn set_state(&self, _state: *mut c_void) -> tresult {
        info!("set_state");
        kResultOk
    }

    unsafe fn get_state(&self, _state: *mut c_void) -> tresult {
        info!("get_state");
        kResultOk
    }

    unsafe fn get_parameter_count(&self) -> i32 {
        info!("get_parameter_count");
        0
    }

    unsafe fn get_parameter_info(&self, _: i32, _: *mut ParameterInfo) -> tresult {
        info!("get_parameter_info");
        kResultFalse
    }

    unsafe fn get_param_string_by_value(
        &self,
        _id: u32,
        _value_normalized: f64,
        _string: *mut TChar,
    ) -> tresult {
        info!("get_param_string_by_value");
        kResultFalse
    }

    unsafe fn get_param_value_by_string(
        &self,
        _id: u32,
        _string: *const TChar,
        _value_normalized: *mut f64,
    ) -> tresult {
        info!("get_param_value_by_string");
        kResultFalse
    }

    unsafe fn normalized_param_to_plain(&self, _id: u32, _value_normalized: f64) -> f64 {
        info!("normalized_param_to_plain");
        0.0
    }

    unsafe fn plain_param_to_normalized(&self, _id: u32, _plain_value: f64) -> f64 {
        info!("plain_param_to_normalized");
        0.0
    }

    unsafe fn get_param_normalized(&self, _id: u32) -> f64 {
        info!("get_param_normalized");
        0.0
    }

    unsafe fn set_param_normalized(&self, _id: u32, _value: f64) -> tresult {
        info!("set_param_normalized");
        kResultOk
    }

    unsafe fn set_component_handler(&self, _handler: *mut c_void) -> tresult {
        info!("set_component_handler");
        kResultOk
    }

    unsafe fn create_view(&self, _name: FIDString) -> *mut c_void {
        info!("Called: AGainController::create_view()");
        null_mut()
    }
}
