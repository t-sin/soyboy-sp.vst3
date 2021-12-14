use std::os::raw::c_void;

use vst3_sys::{
    base::{char16, kResultFalse, kResultOk, tresult, FIDString, TBool},
    gui::{IPlugFrame, IPlugView, ViewRect},
    utils::SharedVstPtr,
    VST3,
};

use crate::vst3::utils;

#[VST3(implements(IPlugView, IPlugFrame))]
pub struct SoyBoyGUI {}

impl SoyBoyGUI {
    pub fn new() -> Box<Self> {
        SoyBoyGUI::allocate()
    }
}

impl IPlugFrame for SoyBoyGUI {
    unsafe fn resize_view(
        &self,
        _view: SharedVstPtr<dyn IPlugView>,
        _new_size: *mut ViewRect,
    ) -> tresult {
        kResultOk
    }
}

impl IPlugView for SoyBoyGUI {
    unsafe fn set_frame(&self, frame: *mut c_void) -> tresult {
        println!("set_frame");
        let frame = frame as *mut _;
        *frame = self as &dyn IPlugFrame;
        kResultOk
    }

    unsafe fn is_platform_type_supported(&self, type_: FIDString) -> tresult {
        println!("is");
        let type_ = utils::fidstring_to_string(type_);

        // TODO: currently supports GUI only on GNU/Linux
        if type_ == "X11EmbedWindowID" {
            println!("aaaaaaaaaaaaa");
            kResultOk
        } else {
            println!("eeeeeeeeeeeeeeeeeee");
            kResultFalse
        }
    }

    unsafe fn attached(&self, _parent: *mut c_void, _type_: FIDString) -> tresult {
        println!("attached");
        kResultOk
    }

    unsafe fn removed(&self) -> tresult {
        println!("aaaaaaaaaaaaa");
        kResultOk
    }
    unsafe fn on_wheel(&self, _distance: f32) -> tresult {
        println!("aaaaaaaaaaaaa");
        kResultOk
    }
    unsafe fn on_key_down(&self, _key: char16, _key_code: i16, _modifiers: i16) -> tresult {
        println!("aaaaaaaaaaaaa");
        kResultOk
    }
    unsafe fn on_key_up(&self, _key: char16, _key_code: i16, _modifiers: i16) -> tresult {
        println!("aaaaaaaaaaaaa");
        kResultOk
    }
    unsafe fn get_size(&self, size: *mut ViewRect) -> tresult {
        (*size).left = 0;
        (*size).top = 0;
        (*size).right = 200;
        (*size).bottom = 200;

        kResultOk
    }
    unsafe fn on_size(&self, _new_size: *mut ViewRect) -> tresult {
        println!("aaaaaaaaaaaaa");
        kResultOk
    }
    unsafe fn on_focus(&self, _state: TBool) -> tresult {
        println!("aaaaaaaaaaaaa");
        kResultOk
    }

    unsafe fn can_resize(&self) -> tresult {
        println!("can_resize");
        kResultFalse
    }
    unsafe fn check_size_constraint(&self, _rect: *mut ViewRect) -> tresult {
        println!("check_size_constraint");
        kResultOk
    }
}
