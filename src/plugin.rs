use log::*;
use std::os::raw::c_void;
use std::ptr::{copy_nonoverlapping, null_mut};

use vst3_com::{sys::GUID, IID};
use vst3_sys::{
    base::{kInvalidArgument, kResultFalse, kResultOk, tresult, FIDString, IPluginBase, TBool},
    vst::{
        AudioBusBuffers, BusDirections, BusFlags, BusInfo, IAudioProcessor, IComponent,
        IEditController, MediaTypes, ParameterInfo, ProcessData, ProcessSetup, RoutingInfo, TChar,
    },
    VST3,
};

use crate::util::wstrcpy;

#[VST3(implements(IComponent, IAudioProcessor, IEditController))]
pub struct GameBoyPlugin {}

impl GameBoyPlugin {
    pub const CID: GUID = GUID {
        data: [
            0xd6, 0x8e, 0x5c, 0xd2, 0x8a, 0x5d, 0x4d, 0xbe, 0xaf, 0xfa, 0x4a, 0x3f, 0x01, 0xfc,
            0x93, 0xd1,
        ],
    };

    pub fn new() -> Box<Self> {
        let gb = GameBoyPlugin::allocate();
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

impl IComponent for GameBoyPlugin {
    unsafe fn get_controller_class_id(&self, _tuid: *mut IID) -> tresult {
        kResultOk
    }

    unsafe fn set_io_mode(&self, _mode: i32) -> tresult {
        kResultOk
    }

    unsafe fn get_bus_count(&self, type_: i32, _dir: i32) -> i32 {
        if type_ == MediaTypes::kAudio as i32 {
            1
        } else {
            0
        }
    }

    unsafe fn get_bus_info(&self, type_: i32, dir: i32, _idx: i32, info: *mut BusInfo) -> tresult {
        if type_ == MediaTypes::kAudio as i32 {
            let info = &mut *info;
            if dir == BusDirections::kInput as i32 {
                info.direction = dir;
                info.bus_type = MediaTypes::kAudio as i32;
                info.channel_count = 2;
                info.flags = BusFlags::kDefaultActive as u32;
                wstrcpy("Audio Input", info.name.as_mut_ptr());
            } else {
                info.direction = dir;
                info.bus_type = MediaTypes::kAudio as i32;
                info.channel_count = 2;
                info.flags = BusFlags::kDefaultActive as u32;
                wstrcpy("Audio Output", info.name.as_mut_ptr());
            }
            kResultOk
        } else {
            kInvalidArgument
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

    unsafe fn can_process_sample_size(&self, _symbolic_sample_size: i32) -> i32 {
        kResultOk
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
        let num_samples = data.num_samples as usize;
        if data.inputs.is_null() || data.outputs.is_null() {
            return kResultOk;
        }
        let inputs: &mut AudioBusBuffers = &mut *data.inputs;
        let outputs: &mut AudioBusBuffers = &mut *data.outputs;
        let num_channels = inputs.num_channels as usize;
        let input_ptr = std::slice::from_raw_parts(inputs.buffers, num_channels);
        let output_ptr = std::slice::from_raw_parts(outputs.buffers, num_channels);
        let sample_size = if data.symbolic_sample_size == 0 { 4 } else { 8 };
        for (i, o) in input_ptr.iter().zip(output_ptr.iter()) {
            copy_nonoverlapping(*i, *o, num_samples * sample_size);
        }
        kResultOk
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
