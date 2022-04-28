use std::cell::RefCell;
use std::collections::HashMap;
use std::os::raw::c_void;
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
};
use std::thread;

use vst3_sys::{
    base::{char16, kResultFalse, kResultOk, tresult, FIDString, TBool},
    gui::{IPlugView, IPlugViewContentScaleSupport, ViewRect},
    vst::IComponentHandler,
    VST3,
};

use crate::gui::{
    EventHandler, GUIEvent, GUIMessage, GUIThread, ParentWindow, SCREEN_HEIGHT, SCREEN_WIDTH,
};
use crate::soyboy::parameters::{ParameterDef, SoyBoyParameter};
use crate::vst3::{common::ControllerConnection, utils};

pub struct VST3EventHandler {
    param_values: Arc<Mutex<HashMap<u32, f64>>>,
    component_handler: Option<Arc<dyn IComponentHandler>>,
}

unsafe impl Send for VST3EventHandler {}
unsafe impl Sync for VST3EventHandler {}

impl VST3EventHandler {
    fn new(
        param_values: Arc<Mutex<HashMap<u32, f64>>>,
        component_handler: Option<Arc<dyn IComponentHandler>>,
    ) -> Self {
        Self {
            param_values,
            component_handler,
        }
    }
}

impl EventHandler for VST3EventHandler {
    fn change_parameter(&self, p: SoyBoyParameter, value_normalized: f64) {
        if let Some(ref handler) = self.component_handler {
            let p = p as u32;

            unsafe {
                handler.begin_edit(p);
                handler.perform_edit(p, value_normalized);
                handler.end_edit(p);
            }

            self.param_values
                .lock()
                .unwrap()
                .insert(p, value_normalized);
        }
    }
}

#[VST3(implements(IPlugView, IPlugViewContentScaleSupport))]
pub struct SoyBoyVST3GUI {
    event_handler: Arc<VST3EventHandler>,
    scale_factor: RefCell<f32>,
    handle: RefCell<Option<thread::JoinHandle<()>>>,
    sender: RefCell<Option<Sender<GUIMessage>>>,
    param_defs: HashMap<SoyBoyParameter, ParameterDef>,
    param_values: Arc<Mutex<HashMap<u32, f64>>>,
    plugin_event_recv: RefCell<Option<Receiver<GUIEvent>>>,
    controller_connection: Arc<ControllerConnection>,
}

impl SoyBoyVST3GUI {
    pub fn new(
        component_handler: Option<Arc<dyn IComponentHandler>>,
        param_defs: HashMap<SoyBoyParameter, ParameterDef>,
        param_values: Arc<Mutex<HashMap<u32, f64>>>,
        plugin_event_recv: Receiver<GUIEvent>,
        controller_connection: Arc<ControllerConnection>,
    ) -> Box<Self> {
        let handler = Arc::new(VST3EventHandler::new(
            param_values.clone(),
            component_handler,
        ));
        let scale_factor = RefCell::new(1.0);
        let handle = RefCell::new(None);
        let sender = RefCell::new(None);
        let plugin_event_recv = RefCell::new(Some(plugin_event_recv));

        SoyBoyVST3GUI::allocate(
            handler,
            scale_factor,
            handle,
            sender,
            param_defs,
            param_values,
            plugin_event_recv,
            controller_connection,
        )
    }

    fn start_gui(&self, parent: ParentWindow) {
        let param_defs = self.param_defs.clone();
        let param_values = self.param_values.clone();
        let event_handler = self.event_handler.clone();
        let controller_connection = self.controller_connection.clone();

        let (send, resv) = channel();
        let recv = Arc::new(Mutex::new(resv));
        (*self.sender.borrow_mut()) = Some(send);

        let plugin_event_recv = self.plugin_event_recv.replace(None);

        let handle = thread::spawn(move || {
            GUIThread::run_loop(
                parent,
                param_defs,
                param_values,
                event_handler,
                recv,
                plugin_event_recv.unwrap(),
                controller_connection,
            );
        });
        *self.handle.borrow_mut() = Some(handle);
    }
}

impl IPlugViewContentScaleSupport for SoyBoyVST3GUI {
    unsafe fn set_scale_factor(&self, scale_factor: f32) -> tresult {
        #[cfg(debug_assertions)]
        println!(
            "IPlugViewContentScaleSupport::set_scale_factor({})",
            scale_factor
        );

        (*self.scale_factor.borrow_mut()) = scale_factor;
        kResultOk
    }
}

impl IPlugView for SoyBoyVST3GUI {
    unsafe fn is_platform_type_supported(&self, type_: FIDString) -> tresult {
        #[cfg(debug_assertions)]
        println!("IPlugView::is_platform_type_supported()");

        let type_ = utils::fidstring_to_string(type_);

        if type_ == "X11EmbedWindowID" || type_ == "HWND" {
            kResultOk
        } else {
            kResultFalse
        }
    }

    unsafe fn attached(&self, parent: *mut c_void, type_: FIDString) -> tresult {
        #[cfg(debug_assertions)]
        println!("IPlugView::attached()");

        let type_ = utils::fidstring_to_string(type_);

        if type_ == "X11EmbedWindowID" || type_ == "HWND" {
            let parent = ParentWindow(parent);
            self.start_gui(parent);
            kResultOk
        } else {
            kResultFalse
        }
    }

    unsafe fn removed(&self) -> tresult {
        #[cfg(debug_assertions)]
        println!("IPlugView::removed()");

        let old_handle = self.handle.replace(None);
        let _ = (*self.sender.borrow())
            .as_ref()
            .unwrap()
            .send(GUIMessage::Terminate);

        #[cfg(debug_assertions)]
        println!("sended terminate.");

        #[allow(unused_variables)]
        if let Some(handle) = old_handle {
            let res = handle.join();
            #[cfg(debug_assertions)]
            println!("joined: {:?}", res);
        }

        let _ = self.sender.replace(None);
        kResultOk
    }

    unsafe fn on_wheel(&self, _distance: f32) -> tresult {
        #[cfg(debug_assertions)]
        println!("IPlugView::on_wheel()");

        kResultOk
    }

    unsafe fn on_key_down(&self, _key: char16, _key_code: i16, _modifiers: i16) -> tresult {
        #[cfg(debug_assertions)]
        println!("IPlugView::on_key_down()");

        kResultOk
    }

    unsafe fn on_key_up(&self, _key: char16, _key_code: i16, _modifiers: i16) -> tresult {
        #[cfg(debug_assertions)]
        println!("IPlugView::on_key_up()");

        kResultOk
    }

    unsafe fn get_size(&self, size: *mut ViewRect) -> tresult {
        #[cfg(debug_assertions)]
        println!("IPlugView::get_size()");

        (*size).left = 0;
        (*size).top = 0;
        (*size).right = SCREEN_WIDTH as i32;
        (*size).bottom = SCREEN_HEIGHT as i32;
        kResultOk
    }

    unsafe fn on_size(&self, _new_size: *mut ViewRect) -> tresult {
        #[cfg(debug_assertions)]
        println!("IPlugView::on_size()");

        kResultOk
    }

    unsafe fn on_focus(&self, _state: TBool) -> tresult {
        #[cfg(debug_assertions)]
        println!("IPlugView::on_focus()");

        kResultOk
    }
    unsafe fn set_frame(&self, _frame: *mut c_void) -> tresult {
        #[cfg(debug_assertions)]
        println!("IPlugView::set_frame()");

        // SoyBoy-SP does not allow GUI resizing so does not implement IPlugFrame inteface.
        // If you need to implement IPlugFrame in Rust, you should implement IPlugFrame not on &self,
        // because if you passed &self as IPlugFrame as a raw pointer (with like `Box::into_raw()`)
        // you cannot drop this object.
        kResultOk
    }

    unsafe fn can_resize(&self) -> tresult {
        #[cfg(debug_assertions)]
        println!("IPlugView::can_resize()");

        kResultFalse
    }

    unsafe fn check_size_constraint(&self, _rect: *mut ViewRect) -> tresult {
        #[cfg(debug_assertions)]
        println!("IPlugView::check_size_constraint()");

        kResultOk
    }
}
