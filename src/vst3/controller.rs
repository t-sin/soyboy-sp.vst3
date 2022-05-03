use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::mem;
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
};

use bincode::Options;

use vst3_com::{interfaces::IUnknown, sys::GUID};
use vst3_sys::{
    base::{
        kInvalidArgument, kResultFalse, kResultOk, kResultTrue, tresult, FIDString, IBStream,
        IPluginBase,
    },
    gui::IPlugView,
    utils::SharedVstPtr,
    vst::{
        kRootUnitId, CtrlNumber, IComponentHandler, IConnectionPoint, IEditController, IMessage,
        IMidiMapping, IUnitInfo, ParamID, ParameterFlags, ParameterInfo, ProgramListInfo, TChar,
        UnitInfo,
    },
    VstPtr, VST3,
};

use crate::common::{GUIEvent, Vst3Message};
use crate::soyboy::parameters::{Normalizable, ParameterDef, Parametric, SoyBoyParameter};
use crate::vst3::{
    gui::SoyBoyVST3GUI, plugin_config::PluginConfigV01, plugin_data, raw_utils, vst3_utils,
};

#[VST3(implements(IEditController, IUnitInfo, IMidiMapping, IConnectionPoint))]
pub struct SoyBoyController {
    param_defs: HashMap<SoyBoyParameter, ParameterDef>,
    vst3_params: RefCell<HashMap<u32, ParameterInfo>>,
    param_values: Arc<Mutex<HashMap<u32, f64>>>,
    processor: RefCell<Option<Arc<dyn IConnectionPoint>>>,
    component_handler: RefCell<Option<Arc<dyn IComponentHandler>>>,
    context: RefCell<Option<VstPtr<dyn IUnknown>>>,
    gui_sender: Mutex<Option<Sender<GUIEvent>>>,
}

struct Paraminfo<'a> {
    title: &'a str,
    short_title: &'a str,
    unit_name: &'a str,
    step_count: i32,
    default_value: f64,
    flags: i32,
}

impl SoyBoyController {
    pub const CID: GUID = GUID {
        data: plugin_data::VST3_CONTROLLER_CID,
    };

    unsafe fn add_parameter(&self, id: u32, paraminfo: Paraminfo) {
        let mut vst3_params = self.vst3_params.borrow_mut();
        let mut param_vals = self.param_values.lock().unwrap();

        let mut param = raw_utils::make_empty_param_info();
        param.id = id;
        raw_utils::wstrcpy(paraminfo.title, param.title.as_mut_ptr());
        raw_utils::wstrcpy(paraminfo.short_title, param.short_title.as_mut_ptr());
        raw_utils::wstrcpy(paraminfo.unit_name, param.units.as_mut_ptr());
        param.step_count = paraminfo.step_count;
        param.default_normalized_value = paraminfo.default_value;
        param.unit_id = kRootUnitId;
        param.flags = paraminfo.flags;

        (*vst3_params).insert(id, param);
        (*param_vals).insert(id, param.default_normalized_value);
    }

    pub unsafe fn new(param_defs: HashMap<SoyBoyParameter, ParameterDef>) -> Box<SoyBoyController> {
        let vst3_params = RefCell::new(HashMap::new());
        let param_vals = Arc::new(Mutex::new(HashMap::new()));
        let processor = RefCell::new(None);
        let component_handler = RefCell::new(None);
        let context = RefCell::new(None);
        let gui_sender = Mutex::new(None);

        SoyBoyController::allocate(
            param_defs,
            vst3_params,
            param_vals,
            processor,
            component_handler,
            context,
            gui_sender,
        )
    }
}

impl IPluginBase for SoyBoyController {
    unsafe fn initialize(&self, host_context: *mut c_void) -> tresult {
        if host_context.is_null() {
            panic!("host context is null");
        }

        let context: VstPtr<dyn IUnknown> = VstPtr::shared(host_context as *mut _).unwrap();
        let _ = self.context.replace(Some(context));

        let param_defs = self.param_defs.clone();
        for (param, soyboy_param) in param_defs.iter() {
            self.add_parameter(
                *param as u32,
                Paraminfo {
                    title: &soyboy_param.title,
                    short_title: &soyboy_param.short_title,
                    unit_name: &soyboy_param.unit_name,
                    step_count: soyboy_param.step_count,
                    default_value: soyboy_param.default_value,
                    flags: ParameterFlags::kCanAutomate as i32,
                },
            );
        }

        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        #[cfg(debug_assertions)]
        println!("SoyBoyController::terminate()");

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

        let mut config_version: u32 = 0;
        let result = state.read(
            &mut config_version as *mut u32 as *mut c_void,
            mem::size_of::<u32>() as i32,
            null_mut(),
        );

        if result != kResultOk {
            println!("IEditController::set_component_state(): read CONFIG_VERSION failed");
            return kResultFalse;
        }

        match config_version {
            PluginConfigV01::CONFIG_VERSION => {
                let options = bincode::config::DefaultOptions::new()
                    .reject_trailing_bytes()
                    .with_little_endian()
                    .with_fixint_encoding();
                let size = options
                    .serialized_size(&PluginConfigV01::default())
                    .unwrap();
                let mut bytes: Vec<u8> = vec![0; size as usize];

                let result = state.read(bytes.as_mut_ptr() as *mut c_void, size as i32, null_mut());

                if result != kResultOk {
                    println!("IEditController::set_component_state(): cannot read PluginConfigV01");
                    return kResultFalse;
                }

                let decoded = options.deserialize(&bytes[..]);
                if decoded.is_err() {
                    println!(
                        "IEditController::set_component_state(): invalid v01 config: {:?}",
                        decoded
                    );
                    return kResultFalse;
                }

                let config: PluginConfigV01 = decoded.unwrap();
                let mut param_vals = self.param_values.lock().unwrap();
                for param in SoyBoyParameter::iter() {
                    let param_def = self.param_defs.get(&param).unwrap();
                    let value = config.get_param(&param);
                    param_vals.insert(param as u32, param_def.normalize(value));
                }
            }
            _ => {
                println!("IEditController::set_component_state(): unsupported VST3 state");
                return kResultFalse;
            }
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
        if let Ok(param) = SoyBoyParameter::try_from(id) {
            if let Some(p) = self.param_defs.get(&param) {
                raw_utils::tcharcpy(&p.format(value_normalized), string)
            } else {
                return kResultFalse;
            }
        }

        kResultOk
    }

    unsafe fn get_param_value_by_string(
        &self,
        id: u32,
        string: *const TChar,
        value_normalized: *mut f64,
    ) -> tresult {
        if let Ok(param) = SoyBoyParameter::try_from(id) {
            if let Some(p) = self.param_defs.get(&param) {
                if let Some(v) = p.parse(&raw_utils::tchar_to_string(string)) {
                    *value_normalized = v;
                } else {
                    return kResultFalse;
                }
            } else {
                return kResultFalse;
            }
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
        handler: SharedVstPtr<dyn IComponentHandler>,
    ) -> tresult {
        #[cfg(debug_assertion)]
        println!("IEditController::set_component_handler()");

        if let Some(handler) = handler.upgrade() {
            (*self.component_handler.borrow_mut()) = Some(Arc::new(handler));
            kResultOk
        } else {
            kResultFalse
        }
    }

    unsafe fn create_view(&self, name: FIDString) -> *mut c_void {
        if raw_utils::fidstring_to_string(name) == "editor" {
            #[cfg(debug_assertions)]
            println!("IEditController::create_view()");

            let (send, recv) = channel::<GUIEvent>();
            *self.gui_sender.lock().unwrap() = Some(send);

            let context = self.context.borrow();
            let context = context.as_ref().unwrap();
            let host = Arc::new(vst3_utils::get_host_app(context));

            let conn = vst3_utils::ControllerConnection::new(
                self.processor.borrow().as_ref().unwrap().clone(),
                host,
            );

            conn.send_message(Vst3Message::WaveTableRequested);

            let gui = SoyBoyVST3GUI::new(
                self.component_handler.borrow().clone(),
                self.param_defs.clone(),
                self.param_values.clone(),
                recv,
                Arc::new(Mutex::new(conn)),
            );

            let gui = Box::into_raw(gui) as *mut dyn IPlugView as *mut c_void;
            #[cfg(debug_assertions)]
            println!("IEditController::create_view(): casted self.gui into *mut c_void");

            gui
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

impl IConnectionPoint for SoyBoyController {
    unsafe fn connect(&self, other: SharedVstPtr<dyn IConnectionPoint>) -> tresult {
        let processor = other.upgrade().unwrap();
        let _ = self.processor.replace(Some(Arc::new(processor)));
        kResultOk
    }

    unsafe fn disconnect(&self, _other: SharedVstPtr<dyn IConnectionPoint>) -> tresult {
        let _ = self.processor.replace(None);
        kResultOk
    }

    unsafe fn notify(&self, message: SharedVstPtr<dyn IMessage>) -> tresult {
        if let Some(sender) = &*self.gui_sender.lock().unwrap() {
            match Vst3Message::from_message(&message) {
                Some(Vst3Message::NoteOn) => {
                    let _ = sender.send(GUIEvent::NoteOn);
                }
                Some(Vst3Message::WaveTableData(table)) => {
                    let _ = sender.send(GUIEvent::WaveTableData(table));
                }
                Some(Vst3Message::WaveformData(wf)) => {
                    let _ = sender.send(GUIEvent::WaveformData(wf));
                }
                _ => (),
            }
        }

        kResultOk
    }
}
