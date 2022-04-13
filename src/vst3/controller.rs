use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::mem;
use std::ops::DerefMut;
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};

use vst3_com::sys::GUID;
use vst3_sys::{
    base::{
        kInvalidArgument, kResultFalse, kResultOk, kResultTrue, tresult, FIDString, IBStream,
        IPluginBase,
    },
    gui::IPlugView,
    utils::SharedVstPtr,
    vst::{
        kRootUnitId, CtrlNumber, IComponentHandler, IEditController, IMidiMapping, IUnitInfo,
        ParamID, ParameterFlags, ParameterInfo, ProgramListInfo, TChar, UnitInfo,
    },
    VST3,
};

use crate::soyboy::parameters::{Normalizable, ParameterDef, SoyBoyParameter};
use crate::vst3::{gui::SoyBoyGUI, plugin_data, utils};

#[VST3(implements(IEditController, IUnitInfo, IMidiMapping))]
pub struct SoyBoyController {
    param_defs: HashMap<SoyBoyParameter, ParameterDef>,
    vst3_params: RefCell<HashMap<u32, ParameterInfo>>,
    param_values: Arc<Mutex<HashMap<u32, f64>>>,
    gui: RefCell<Box<SoyBoyGUI>>,
}

impl SoyBoyController {
    pub const CID: GUID = GUID {
        data: plugin_data::VST3_CONTROLLER_CID,
    };

    unsafe fn add_parameter(
        &self,
        id: u32,
        title: &str,
        short_title: &str,
        units: &str,
        step_count: i32,
        default_value: f64,
        flags: i32,
    ) {
        let mut vst3_params = self.vst3_params.borrow_mut();
        let mut param_vals = self.param_values.lock().unwrap();

        let mut param = utils::make_empty_param_info();
        param.id = id;
        utils::wstrcpy(title, param.title.as_mut_ptr());
        utils::wstrcpy(short_title, param.short_title.as_mut_ptr());
        utils::wstrcpy(units, param.units.as_mut_ptr());
        param.step_count = step_count;
        param.default_normalized_value = default_value;
        param.unit_id = kRootUnitId;
        param.flags = flags;

        (*vst3_params).insert(id, param);
        (*param_vals).insert(id, param.default_normalized_value);
    }

    pub unsafe fn new(param_defs: HashMap<SoyBoyParameter, ParameterDef>) -> Box<SoyBoyController> {
        let vst3_params = RefCell::new(HashMap::new());
        let param_vals = Arc::new(Mutex::new(HashMap::new()));
        let gui = RefCell::new(SoyBoyGUI::new(param_defs.clone()));

        SoyBoyController::allocate(param_defs, vst3_params, param_vals, gui)
    }
}

impl IPluginBase for SoyBoyController {
    unsafe fn initialize(&self, _host_context: *mut c_void) -> tresult {
        let param_defs = self.param_defs.clone();
        for (param, soyboy_param) in param_defs.iter() {
            self.add_parameter(
                *param as u32,
                &soyboy_param.title,
                &soyboy_param.short_title,
                &soyboy_param.unit_name,
                soyboy_param.step_count,
                soyboy_param.default_value,
                ParameterFlags::kCanAutomate as i32,
            );
        }

        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}

impl IMidiMapping for SoyBoyController {
    unsafe fn get_midi_controller_assignment(
        &self,
        _bus_index: i32,
        _channel: i16,
        midi_cc_number: CtrlNumber,
        param_id: *mut ParamID,
    ) -> tresult {
        match midi_cc_number {
            // kPitchBend
            // cf.
            // - https://www.utsbox.com/?p=1109
            // - https://steinbergmedia.github.io/vst3_doc/vstinterfaces/namespaceSteinberg_1_1Vst.html#a70ee68a13248febed5047cfa0fddf4e6
            129 => {
                *param_id = SoyBoyParameter::PitchBend as u32;
                kResultTrue
            }
            _ => kResultFalse,
        }
    }
}

impl IEditController for SoyBoyController {
    unsafe fn set_component_state(&self, state: SharedVstPtr<dyn IBStream>) -> tresult {
        if state.is_null() {
            return kResultFalse;
        }

        let state = state.upgrade();
        if state.is_none() {
            return kResultFalse;
        }
        let state = state.unwrap();

        let mut num_bytes_read = 0;
        for param in SoyBoyParameter::iter() {
            let mut value = 0.0;
            let ptr = &mut value as *mut f64 as *mut c_void;

            state.read(ptr, mem::size_of::<f64>() as i32, &mut num_bytes_read);
            self.param_values
                .lock()
                .unwrap()
                .insert(param as u32, value);
        }

        kResultOk
    }

    unsafe fn set_state(&self, _state: SharedVstPtr<dyn IBStream>) -> tresult {
        kResultOk
    }

    unsafe fn get_state(&self, _state: SharedVstPtr<dyn IBStream>) -> tresult {
        kResultOk
    }

    unsafe fn get_parameter_count(&self) -> i32 {
        self.vst3_params.borrow().len() as i32
    }

    unsafe fn get_parameter_info(&self, id: i32, vst3_params: *mut ParameterInfo) -> tresult {
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
        match SoyBoyParameter::try_from(id) {
            Ok(param) => {
                if let Some(p) = self.param_defs.get(&param) {
                    utils::tcharcpy(&p.format(value_normalized), string)
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
        match SoyBoyParameter::try_from(id) {
            Ok(param) => {
                if let Some(p) = self.param_defs.get(&param) {
                    if let Some(v) = p.parse(&utils::tchar_to_string(string)) {
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
        match SoyBoyParameter::try_from(id) {
            Ok(param) => {
                if let Some(p) = self.param_defs.get(&param) {
                    p.denormalize(value_normalized)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    unsafe fn plain_param_to_normalized(&self, id: u32, value_plain: f64) -> f64 {
        match SoyBoyParameter::try_from(id) {
            Ok(param) => {
                if let Some(p) = self.param_defs.get(&param) {
                    p.normalize(value_plain)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    unsafe fn get_param_normalized(&self, id: u32) -> f64 {
        match self.param_values.lock().unwrap().get(&id) {
            Some(val) => *val,
            _ => 0.0,
        }
    }

    unsafe fn set_param_normalized(&self, id: u32, value: f64) -> tresult {
        match self.param_values.lock().unwrap().insert(id, value) {
            Some(_) => kResultTrue,
            _ => kResultFalse,
        }
    }

    unsafe fn set_component_handler(
        &self,
        _handler: SharedVstPtr<dyn IComponentHandler>,
    ) -> tresult {
        kResultOk
    }

    unsafe fn create_view(&self, name: FIDString) -> *mut c_void {
        if utils::fidstring_to_string(name) == "editor" {
            println!("IEditController::create_view()");

            // MEMO: When re-open the plugin window, the VST3 host calls this IEditController::create_view() but
            //       self.gui have did borrow_mut() and casted as *mut c_void in previous call, it goes non-safe,
            //       so we make a fresh GUI object for a new IEditController::create_view() call.
            (*self.gui.borrow_mut()) = SoyBoyGUI::new(self.param_defs.clone());

            // MEMO: When I implement IPlugView as IEditController itself but self in here
            //       is not mutable, so I wrote a complex casting and it does not works
            //       (the VST host doesn't seem it as IPlugVIew)
            //       (because of mutabillity difference? or complex casting? I don't know)
            //
            //       So I decided IPlugView as new object (in gui.rs). In this way, the VST3 host
            //       recognizes IPlugView and proceeds GUI initialization sequence.
            self.gui.borrow_mut().deref_mut().as_mut() as *const dyn IPlugView as *mut c_void
        } else {
            null_mut()
        }
    }
}

impl IUnitInfo for SoyBoyController {
    unsafe fn get_unit_count(&self) -> i32 {
        1
    }

    unsafe fn get_unit_info(&self, _unit_index: i32, _info: *mut UnitInfo) -> i32 {
        kResultFalse
    }

    unsafe fn get_program_list_count(&self) -> i32 {
        0
    }

    unsafe fn get_program_list_info(&self, _list_index: i32, _info: *mut ProgramListInfo) -> i32 {
        kResultFalse
    }

    unsafe fn get_program_name(&self, _list_id: i32, _program_index: i32, _name: *mut u16) -> i32 {
        kResultFalse
    }

    unsafe fn get_program_info(
        &self,
        _list_id: i32,
        _program_index: i32,
        _attribute_id: *const u8,
        _attribute_value: *mut u16,
    ) -> i32 {
        kResultFalse
    }

    unsafe fn has_program_pitch_names(&self, _id: i32, _index: i32) -> i32 {
        kResultFalse
    }

    unsafe fn get_program_pitch_name(
        &self,
        _id: i32,
        _index: i32,
        _pitch: i16,
        _name: *mut u16,
    ) -> i32 {
        kResultFalse
    }

    unsafe fn get_selected_unit(&self) -> i32 {
        0
    }

    unsafe fn select_unit(&self, _id: i32) -> i32 {
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
        kResultFalse
    }

    unsafe fn set_unit_program_data(
        &self,
        _list_or_unit: i32,
        _program_index: i32,
        _data: SharedVstPtr<dyn IBStream>,
    ) -> i32 {
        kResultFalse
    }
}
