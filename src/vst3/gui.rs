use std::cell::RefCell;
use std::os::raw::c_void;
use std::thread;

use vst3_sys::{
    base::{char16, kResultFalse, kResultOk, tresult, FIDString, TBool},
    gui::{IPlugFrame, IPlugView, ViewRect},
    utils::SharedVstPtr,
    VST3,
};

use crate::vst3::utils;

use egui::Checkbox;
use egui_backend::sdl2::video::GLProfile;
use egui_backend::{egui, gl, sdl2};
use egui_backend::{sdl2::event::Event, DpiScaling, ShaderVersion};
use std::time::Instant;
// Alias the backend to something less mouthful
use egui_sdl2_gl as egui_backend;
use sdl2::video::SwapInterval;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

struct EguiApp {
    handle: Option<thread::JoinHandle<()>>,
}

impl EguiApp {
    fn new() -> Self {
        Self { handle: None }
    }

    fn start(&mut self) {
        let handle = thread::spawn(|| {
            let sdl_context = sdl2::init().unwrap();
            let video_subsystem = sdl_context.video().unwrap();
            let gl_attr = video_subsystem.gl_attr();
            gl_attr.set_context_profile(GLProfile::Core);
            // On linux, OpenGL ES Mesa driver 22.0.0+ can be used like so:
            // gl_attr.set_context_profile(GLProfile::GLES);

            gl_attr.set_double_buffer(true);
            gl_attr.set_multisample_samples(4);

            let window = video_subsystem
                .window(
                    "Demo: Egui backend for SDL2 + GL",
                    SCREEN_WIDTH,
                    SCREEN_HEIGHT,
                )
                .opengl()
                .resizable()
                .build()
                .unwrap();

            // Create a window context
            let _ctx = window.gl_create_context().unwrap();

            let mut enable_vsync = false;
            let mut quit = false;
            let mut slider = 0.0;
            let start_time = Instant::now();
            let shader_ver = ShaderVersion::Default;

            let sdl_context = window.subsystem().sdl();
            let (mut painter, mut egui_state) =
                egui_backend::with_sdl2(&window, shader_ver.clone(), DpiScaling::Custom(2.0));
            let mut egui_ctx = egui::CtxRef::default();
            let mut event_pump = sdl_context.event_pump().unwrap();

            let mut test_str: String =
                "A text box to write in. Cut, copy, paste commands are available.".to_owned();

            'running: loop {
                if enable_vsync {
                    window
                        .subsystem()
                        .gl_set_swap_interval(SwapInterval::VSync)
                        .unwrap()
                } else {
                    window
                        .subsystem()
                        .gl_set_swap_interval(SwapInterval::Immediate)
                        .unwrap()
                }

                egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
                egui_ctx.begin_frame(egui_state.input.take());

                egui::CentralPanel::default().show(&egui_ctx, |ui| {
                    ui.label(" ");
                    ui.text_edit_multiline(&mut test_str);
                    ui.label(" ");
                    ui.add(egui::Slider::new(&mut slider, 0.0..=50.0).text("Slider"));
                    ui.label(" ");
                    ui.add(Checkbox::new(&mut enable_vsync, "Reduce CPU Usage?"));
                    ui.separator();
                    if ui.button("Quit?").clicked() {
                        quit = true;
                    }
                });

                let (egui_output, paint_cmds) = egui_ctx.end_frame();
                // Process ouput
                egui_state.process_output(&window, &egui_output);

                // For default dpi scaling only, Update window when the size of resized window is very small (to avoid egui::CentralPanel distortions).
                // if egui_ctx.used_size() != painter.screen_rect.size() {
                //     println!("resized.");
                //     let _size = egui_ctx.used_size();
                //     let (w, h) = (_size.x as u32, _size.y as u32);
                //     window.set_size(w, h).unwrap();
                // }

                let paint_jobs = egui_ctx.tessellate(paint_cmds);

                // An example of how OpenGL can be used to draw custom stuff with egui
                // overlaying it:
                // First clear the background to something nice.
                unsafe {
                    // Clear the screen to green
                    gl::ClearColor(0.3, 0.6, 0.3, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }

                if !egui_output.needs_repaint {
                    if let Some(event) = event_pump.wait_event_timeout(5) {
                        match event {
                            Event::Quit { .. } => break 'running,
                            _ => {
                                // Process input event
                                egui_state.process_input(&window, event, &mut painter);
                            }
                        }
                    }
                } else {
                    painter.paint_jobs(None, paint_jobs, &egui_ctx.font_image());
                    window.gl_swap_window();
                    for event in event_pump.poll_iter() {
                        match event {
                            Event::Quit { .. } => break 'running,
                            _ => {
                                // Process input event
                                egui_state.process_input(&window, event, &mut painter);
                            }
                        }
                    }
                }

                if quit {
                    break;
                }
            }
        });

        self.handle = Some(handle);
    }
}

#[VST3(implements(IPlugView, IPlugFrame))]
pub struct SoyBoyGUI {
    app: RefCell<EguiApp>,
}

impl SoyBoyGUI {
    pub fn new() -> Box<Self> {
        let app = RefCell::new(EguiApp::new());
        SoyBoyGUI::allocate(app)
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

    #[cfg(not(target_arch = "wasm32"))]
    unsafe fn attached(&self, _parent: *mut c_void, _type_: FIDString) -> tresult {
        println!("attached");
        self.app.borrow_mut().start();
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
