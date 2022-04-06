use std::cell::RefCell;
use std::os::raw::c_void;
use std::thread;
use std::time::Instant;

use vst3_sys::{
    base::{char16, kResultFalse, kResultOk, tresult, FIDString, TBool},
    gui::{IPlugFrame, IPlugView, ViewRect},
    utils::SharedVstPtr,
    VST3,
};

use egui_backend::sdl2::video::GLProfile;
use egui_backend::{egui, gl, sdl2};
use egui_backend::{sdl2::event::Event, DpiScaling, ShaderVersion};
use egui_sdl2_gl as egui_backend;
use sdl2::{
    video::{GLContext, SwapInterval, Window},
    EventPump,
};

use crate::vst3::utils;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

struct GUIThread {
    // SoyBoy specific
    slider: f64,
    // window stuff
    quit: bool,
    start_time: Instant,
    // egui stuff
    window: Window,
    gl_context: GLContext,
    egui_context: egui::CtxRef,
    egui_state: egui_backend::EguiStateHandler,
    event_pump: EventPump,
    painter: egui_backend::painter::Painter,
}

// from this code originally:
//   https://github.com/ArjunNair/egui_sdl2_gl/blob/main/examples/basic.rs
impl GUIThread {
    fn setup() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        // On linux, OpenGL ES Mesa driver 22.0.0+ can be used like so:
        // gl_attr.set_context_profile(GLProfile::GLES);

        gl_attr.set_double_buffer(true);
        gl_attr.set_multisample_samples(4);

        let window = video_subsystem
            .window("SoyBoy SP v0.5.x", SCREEN_WIDTH, SCREEN_HEIGHT)
            .opengl()
            .resizable()
            .build()
            .unwrap();

        // Create a window context
        let gl_context = window.gl_create_context().unwrap();
        let shader_ver = ShaderVersion::Default;

        let sdl_context = window.subsystem().sdl();
        let (painter, egui_state) =
            egui_backend::with_sdl2(&window, shader_ver.clone(), DpiScaling::Custom(2.0));
        let egui_context = egui::CtxRef::default();
        let event_pump = sdl_context.event_pump().unwrap();

        GUIThread {
            slider: 0.0,
            quit: false,
            window: window,
            start_time: Instant::now(),
            gl_context: gl_context,
            egui_context: egui_context,
            egui_state: egui_state,
            event_pump: event_pump,
            painter: painter,
        }
    }

    fn update(&mut self) {
        let _ = self.window.gl_make_current(&self.gl_context);
        self.window
            .subsystem()
            .gl_set_swap_interval(SwapInterval::Immediate)
            .unwrap();
        self.egui_state.input.time = Some(self.start_time.elapsed().as_secs_f64());
        self.egui_context.begin_frame(self.egui_state.input.take());
    }

    fn construct_gui(&mut self) {
        let mut test_str: String = "SoyBoooooooooooooooooooy".to_owned();

        egui::CentralPanel::default().show(&self.egui_context, |ui| {
            ui.label(" ");
            ui.text_edit_multiline(&mut test_str);
            ui.label(" ");
            ui.add(egui::Slider::new(&mut self.slider, 0.0..=50.0).text("Slider"));
            ui.label(" ");
        });
    }

    fn draw(&mut self) {
        let (egui_output, paint_cmds) = self.egui_context.end_frame();
        self.egui_state.process_output(&self.window, &egui_output);

        let paint_jobs = self.egui_context.tessellate(paint_cmds);

        unsafe {
            // Clear the screen to green
            gl::ClearColor(0.3, 0.6, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        if !egui_output.needs_repaint {
            if let Some(event) = self.event_pump.wait_event_timeout(5) {
                match event {
                    Event::Quit { .. } => {
                        self.quit = true;
                    }
                    _ => {
                        // Process input event
                        self.egui_state
                            .process_input(&self.window, event, &mut self.painter);
                    }
                }
            }
        } else {
            self.painter
                .paint_jobs(None, paint_jobs, &self.egui_context.font_image());
            self.window.gl_swap_window();
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => {
                        self.quit = true;
                    }
                    _ => {
                        // Process input event
                        self.egui_state
                            .process_input(&self.window, event, &mut self.painter);
                    }
                }
            }
        }
    }

    fn run_loop() {
        let mut thread = GUIThread::setup();

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

    fn start_gui(&self) {
        let handle = thread::spawn(|| {
            GUIThread::run_loop();
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

    unsafe fn attached(&self, _parent: *mut c_void, _type_: FIDString) -> tresult {
        println!("IPlugView::attached()");
        self.start_gui();
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
