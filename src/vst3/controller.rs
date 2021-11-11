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

use crate::soyboy::Parameter;
use crate::vst3::{
    parameters::{Normalizable, SoyBoyParameter},
    plugin_data, util,
};

#[VST3(implements(IEditController, IUnitInfo))]
pub struct SoyBoyController {
    soyboy_params: HashMap<Parameter, SoyBoyParameter>,
    vst3_params: RefCell<HashMap<u32, ParameterInfo>>,
    param_values: RefCell<HashMap<u32, f64>>,
}

impl SoyBoyController {
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
        let mut vst3_params = self.vst3_params.borrow_mut();
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

        (*vst3_params).insert(id, param);
        (*param_vals).insert(id, param.default_normalized_value);
    }

    pub unsafe fn new(soyboy_params: HashMap<Parameter, SoyBoyParameter>) -> Box<SoyBoyController> {
        let vst3_params = RefCell::new(HashMap::new());
        let param_vals = RefCell::new(HashMap::new());

        let mut controller = SoyBoyController::allocate(soyboy_params, vst3_params, param_vals);

        let soyboy_params = controller.soyboy_params.clone();
        for (param, soyboy_param) in soyboy_params.iter() {
            controller.add_parameter(
                *param as u32,
                &soyboy_param.title,
                &soyboy_param.short_title,
                &soyboy_param.unit_name,
                soyboy_param.step_count,
                soyboy_param.default_value,
                ParameterFlags::kCanAutomate as i32,
            );
        }

        controller
    }
}

impl IPluginBase for SoyBoyController {
    unsafe fn initialize(&self, _host_context: *mut c_void) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}

impl IEditController for SoyBoyController {
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
        self.vst3_params.borrow().len() as i32
    }

    unsafe fn get_parameter_info(&self, id: i32, vst3_params: *mut ParameterInfo) -> tresult {
        info!("get_parameter_info");

        let id = id as u32;

        if let Some(param) = self.vst3_params.borrow().get(&id) {
            *vst3_params = *param;

            kResultOk
        } else {
            kInvalidArgument
        }
    }

    unsafe fn get_param_string_by_value(
        &self,
        id: u32,
        value_normalized: f64,
        string: *mut TChar,
    ) -> tresult {
        match Parameter::try_from(id) {
            Ok(param) => {
                if let Some(p) = self.soyboy_params.get(&param) {
                    util::tcharcpy(&p.format(value_normalized), string)
                } else {
                    return kResultFalse;
                }
            }
            _ => (),
        }

        kResultOk
    }

    unsafe fn get_param_value_by_string(
        &self,
        id: u32,
        string: *const TChar,
        value_normalized: *mut f64,
    ) -> tresult {
        match Parameter::try_from(id) {
            Ok(param) => {
                if let Some(p) = self.soyboy_params.get(&param) {
                    if let Some(v) = p.parse(&util::tchar_to_string(string)) {
                        *value_normalized = v;
                    } else {
                        return kResultFalse;
                    }
                } else {
                    return kResultFalse;
                }
            }
            _ => (),
        }
        kResultOk
    }

    unsafe fn normalized_param_to_plain(&self, id: u32, value_normalized: f64) -> f64 {
        match Parameter::try_from(id) {
            Ok(param) => {
                if let Some(p) = self.soyboy_params.get(&param) {
                    p.denormalize(value_normalized)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    unsafe fn plain_param_to_normalized(&self, id: u32, value_plain: f64) -> f64 {
        match Parameter::try_from(id) {
            Ok(param) => {
                if let Some(p) = self.soyboy_params.get(&param) {
                    p.normalize(value_plain)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
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

impl IUnitInfo for SoyBoyController {
    unsafe fn get_unit_count(&self) -> i32 {
        info!("Called: AGainController::get_unit_count()");

        1
    }

    unsafe fn get_unit_info(&self, _unit_index: i32, _info: *mut UnitInfo) -> i32 {
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
