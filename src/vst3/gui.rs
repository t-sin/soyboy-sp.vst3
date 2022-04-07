use std::cell::RefCell;
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::thread;
use std::time::Instant;

use vst3_sys::{
    base::{char16, kResultFalse, kResultOk, tresult, FIDString, TBool},
    gui::{IPlugFrame, IPlugView, ViewRect},
    utils::SharedVstPtr,
    VST3,
};

use crate::vst3::utils;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

struct ParentWindow(*mut c_void);
unsafe impl Send for ParentWindow {}
unsafe impl Sync for ParentWindow {}

struct GUIThread {
    // SoyBoy specific
    slider: f64,
    // window stuff
    quit: bool,
    // egui stuff
}

impl GUIThread {
    fn setup(_parent: ParentWindow) -> Self {
        GUIThread {
            slider: 0.0,
            quit: false,
        }
    }

    fn update(&mut self) {}

    fn construct_gui(&mut self) {}

    fn draw(&mut self) {}

    fn run_loop(parent: ParentWindow) {
        let mut thread = GUIThread::setup(parent);

        loop {
            thread.update();
            thread.construct_gui();
            thread.draw();

            if thread.quit {
                break;
            }
        }
    }
}

#[VST3(implements(IPlugView, IPlugFrame))]
pub struct SoyBoyGUI {
    handle: RefCell<Option<thread::JoinHandle<()>>>,
}

impl SoyBoyGUI {
    pub fn new() -> Box<Self> {
        let handle = RefCell::new(None);

        SoyBoyGUI::allocate(handle)
    }

    fn start_gui(&self, parent: ParentWindow) {
        let handle = thread::spawn(move || {
            GUIThread::run_loop(parent);
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
        println!("IPlugFrame::reqise_view()");
        (*new_size).left = 0;
        (*new_size).top = 0;
        (*new_size).right = SCREEN_WIDTH as i32;
        (*new_size).bottom = SCREEN_HEIGHT as i32;

        kResultOk
    }
}

impl IPlugView for SoyBoyGUI {
    unsafe fn is_platform_type_supported(&self, type_: FIDString) -> tresult {
        println!("IPlugView::is_platform_type_supported()");
        let type_ = utils::fidstring_to_string(type_);

        // TODO: currently supports GUI only on GNU/Linux
        if type_ == "X11EmbedWindowID" {
            kResultOk
        } else {
            kResultFalse
        }
    }

    unsafe fn attached(&self, parent: *mut c_void, _type_: FIDString) -> tresult {
        println!("IPlugView::attached()");
        let parent = ParentWindow(parent);
        self.start_gui(parent);

        kResultOk
    }

    unsafe fn removed(&self) -> tresult {
        println!("IPlugView::removed()");
        kResultOk
    }
    unsafe fn on_wheel(&self, _distance: f32) -> tresult {
        println!("IPlugView::on_wheel()");
        kResultOk
    }
    unsafe fn on_key_down(&self, _key: char16, _key_code: i16, _modifiers: i16) -> tresult {
        println!("IPlugView::on_key_down()");
        kResultOk
    }
    unsafe fn on_key_up(&self, _key: char16, _key_code: i16, _modifiers: i16) -> tresult {
        println!("IPlugView::on_key_up()");
        kResultOk
    }
    unsafe fn get_size(&self, size: *mut ViewRect) -> tresult {
        println!("IPlugView::get_size()");
        (*size).left = 0;
        (*size).top = 0;
        (*size).right = SCREEN_WIDTH as i32;
        (*size).bottom = SCREEN_HEIGHT as i32;
        kResultOk
    }
    unsafe fn on_size(&self, _new_size: *mut ViewRect) -> tresult {
        println!("IPlugView::on_size()");
        kResultOk
    }
    unsafe fn on_focus(&self, _state: TBool) -> tresult {
        println!("IPlugView::on_focus()");
        kResultOk
    }
    unsafe fn set_frame(&self, frame: *mut c_void) -> tresult {
        println!("IPlugView::set_frame()");
        let frame = frame as *mut _;
        *frame = self as &dyn IPlugFrame;
        kResultOk
    }
    unsafe fn can_resize(&self) -> tresult {
        println!("IPlugView::can_resize()");
        kResultOk
    }
    unsafe fn check_size_constraint(&self, _rect: *mut ViewRect) -> tresult {
        println!("IPlugView::check_size_constraint()");
        kResultOk
    }
}
