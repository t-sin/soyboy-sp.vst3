use log::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;

use std::os::raw::c_void;
use std::ptr::null_mut;

use vst3_com::sys::GUID;
use vst3_sys::{
    base::{
        kInvalidArgument, kResultFalse, kResultOk, kResultTrue, tresult, FIDString, IBStream,
        IPluginBase,
    },
    utils::VstPtr,
    vst::{
        kRootUnitId, IEditController, IUnitInfo, ParameterFlags, ParameterInfo, ProgramListInfo,
        TChar, UnitInfo,
    },
    VST3,
};

use crate::vst3::{parameter::PluginParameter, plugin_data, util};

#[VST3(implements(IEditController, IUnitInfo))]
pub struct GameBoyController {
    param_info: RefCell<HashMap<u32, ParameterInfo>>,
    param_values: RefCell<HashMap<u32, f64>>,
}

impl GameBoyController {
    pub const CID: GUID = GUID {
        data: plugin_data::VST3_CONTROLLER_CID,
    };

    unsafe fn add_parameter(
        &mut self,
        id: u32,
        title: &str,
        short_title: &str,
        units: &str,
        step_count: i32,
        default_value: f64,
        flags: i32,
    ) {
        let mut param_info = self.param_info.borrow_mut();
        let mut param_vals = self.param_values.borrow_mut();

        let mut param = util::make_empty_param_info();
        param.id = id;
        util::wstrcpy(title, param.title.as_mut_ptr());
        util::wstrcpy(short_title, param.short_title.as_mut_ptr());
        util::wstrcpy(units, param.units.as_mut_ptr());
        param.step_count = step_count;
        param.default_normalized_value = default_value;
        param.unit_id = kRootUnitId;
        param.flags = flags;

        (*param_info).insert(id, param);
        (*param_vals).insert(id, param.default_normalized_value);
    }

    pub unsafe fn new() -> Box<GameBoyController> {
        let param_info = RefCell::new(HashMap::new());
        let param_vals = RefCell::new(HashMap::new());

        let mut controller = GameBoyController::allocate(param_info, param_vals);

        controller.add_parameter(
            PluginParameter::Param1 as u32,
            "Test Parameter",
            "Test",
            "",
            0,
            0.0,
            ParameterFlags::kCanAutomate as i32,
        );

        controller
    }
}

impl IPluginBase for GameBoyController {
    unsafe fn initialize(&self, _host_context: *mut c_void) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}

impl IEditController for GameBoyController {
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
        self.param_info.borrow().len() as i32
    }

    unsafe fn get_parameter_info(&self, id: i32, param_info: *mut ParameterInfo) -> tresult {
        info!("get_parameter_info");

        let id = id as u32;

        if let Some(param) = self.param_info.borrow().get(&id) {
            *param_info = *param;

            kResultOk
        } else {
            kInvalidArgument
        }
    }

    unsafe fn get_param_string_by_value(
        &self,
        _id: u32,
        _value_normalized: f64,
        _string: *mut TChar,
    ) -> tresult {
        info!("get_param_string_by_value");

        kResultOk
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

    unsafe fn get_param_normalized(&self, id: u32) -> f64 {
        info!("get_param_normalized");
        match self.param_values.borrow().get(&id) {
            Some(val) => *val,
            _ => 0.0,
        }
    }

    unsafe fn set_param_normalized(&self, id: u32, value: f64) -> tresult {
        info!("set_param_normalized");
        match self.param_values.borrow_mut().insert(id, value) {
            Some(_) => kResultTrue,
            _ => kResultFalse,
        }
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

impl IUnitInfo for GameBoyController {
    unsafe fn get_unit_count(&self) -> i32 {
        info!("Called: AGainController::get_unit_count()");

        1
    }

    unsafe fn get_unit_info(&self, unit_index: i32, info: *mut UnitInfo) -> i32 {
        info!("Called: AGainController::get_unit_info()");

        kResultFalse
    }

    unsafe fn get_program_list_count(&self) -> i32 {
        info!("Called: AGainController::get_program_list_count()");

        0
    }

    unsafe fn get_program_list_info(&self, _list_index: i32, _info: *mut ProgramListInfo) -> i32 {
        info!("Called: AGainController::get_program_list_info()");

        kResultFalse
    }

    unsafe fn get_program_name(&self, _list_id: i32, _program_index: i32, _name: *mut u16) -> i32 {
        info!("Called: AGainController::get_program_name()");

        kResultFalse
    }

    unsafe fn get_program_info(
        &self,
        _list_id: i32,
        _program_index: i32,
        _attribute_id: *const u8,
        _attribute_value: *mut u16,
    ) -> i32 {
        info!("Called: AGainController::get_program_info()");

        kResultFalse
    }

    unsafe fn has_program_pitch_names(&self, _id: i32, _index: i32) -> i32 {
        info!("Called: AGainController::has_program_pitch_names()");

        kResultFalse
    }

    unsafe fn get_program_pitch_name(
        &self,
        _id: i32,
        _index: i32,
        _pitch: i16,
        _name: *mut u16,
    ) -> i32 {
        info!("Called: AGainController::get_program_pitch_name()");

        kResultFalse
    }

    unsafe fn get_selected_unit(&self) -> i32 {
        info!("Called: AGainController::get_selected_unit()");
        0
    }

    unsafe fn select_unit(&self, _id: i32) -> i32 {
        info!("Called: AGainController::select_unit()");

        kResultFalse
    }

    unsafe fn get_unit_by_bus(
        &self,
        _type_: i32,
        _dir: i32,
        _index: i32,
        _channel: i32,
        _unit_id: *mut i32,
    ) -> i32 {
        info!("Called: AGainController::set_unit_by_bus()");

        kResultFalse
    }

    unsafe fn set_unit_program_data(
        &self,
        _list_or_unit: i32,
        _program_index: i32,
        _data: VstPtr<dyn IBStream>,
    ) -> i32 {
        info!("Called: AGainController::set_unit_program_data()");

        kResultFalse
    }
}
