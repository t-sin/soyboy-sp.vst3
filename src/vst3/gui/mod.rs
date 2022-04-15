use std::cell::RefCell;
use std::collections::HashMap;
use std::os::raw::c_void;
use std::sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
};
use std::thread;

use vst3_sys::{
    base::{char16, kResultFalse, kResultOk, tresult, FIDString, TBool},
    gui::{IPlugFrame, IPlugView, IPlugViewContentScaleSupport, ViewRect},
    utils::SharedVstPtr,
    VST3,
};

use crate::soyboy::parameters::{ParameterDef, SoyBoyParameter};
use crate::vst3::utils;

mod constants;
mod gui_thread;
mod types;
mod widget;

use constants::*;
use gui_thread::*;
use types::*;

#[VST3(implements(IPlugView, IPlugFrame, IPlugViewContentScaleSupport))]
pub struct SoyBoyGUI {
    scale_factor: RefCell<f32>,
    handle: RefCell<Option<thread::JoinHandle<()>>>,
    sender: RefCell<Option<Sender<GUIMessage>>>,
    param_defs: HashMap<SoyBoyParameter, ParameterDef>,
}

impl SoyBoyGUI {
    pub fn new(param_defs: HashMap<SoyBoyParameter, ParameterDef>) -> Box<Self> {
        let scale_factor = RefCell::new(1.0);
        let handle = RefCell::new(None);
        let sender = RefCell::new(None);

        SoyBoyGUI::allocate(scale_factor, handle, sender, param_defs)
    }

    fn start_gui(&self, parent: ParentWindow) {
        let param_defs = self.param_defs.clone();

        let (send, resv) = channel();
        let recv = Arc::new(Mutex::new(resv));
        (*self.sender.borrow_mut()) = Some(send);

        let handle = thread::spawn(move || {
            GUIThread::run_loop(parent, param_defs, recv);
        });
        *self.handle.borrow_mut() = Some(handle);
    }
}

impl IPlugFrame for SoyBoyGUI {
    unsafe fn resize_view(
        &self,
        _view: SharedVstPtr<dyn IPlugView>,
        new_size: *mut ViewRect,
    ) -> tresult {
        #[cfg(debug_assertions)]
        println!("IPlugFrame::reqise_view()");

        (*new_size).left = 0;
        (*new_size).top = 0;
        (*new_size).right = SCREEN_WIDTH as i32;
        (*new_size).bottom = SCREEN_HEIGHT as i32;

        kResultOk
    }
}

impl IPlugViewContentScaleSupport for SoyBoyGUI {
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

impl IPlugView for SoyBoyGUI {
    unsafe fn is_platform_type_supported(&self, type_: FIDString) -> tresult {
        #[cfg(debug_assertions)]
        println!("IPlugView::is_platform_type_supported()");

        let type_ = utils::fidstring_to_string(type_);

        if type_ == "X11EmbedWindowID" {
            kResultOk
        } else if type_ == "HWND" {
            kResultOk
        } else {
            kResultFalse
        }
    }

    unsafe fn attached(&self, parent: *mut c_void, type_: FIDString) -> tresult {
        #[cfg(debug_assertions)]
        println!("IPlugView::attached()");

        let type_ = utils::fidstring_to_string(type_);

        if type_ == "X11EmbedWindowID" {
            let parent = ParentWindow(parent);
            self.start_gui(parent);
            kResultOk
        } else if type_ == "HWND" {
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
        let res = old_handle.unwrap().join();

        #[cfg(debug_assertions)]
        println!("joined: {:?}", res);

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
    unsafe fn set_frame(&self, frame: *mut c_void) -> tresult {
        #[cfg(debug_assertions)]
        println!("IPlugView::set_frame()");

        let frame = frame as *mut _;
        *frame = self as &dyn IPlugFrame;
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
