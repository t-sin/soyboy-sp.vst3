use std::cell::RefCell;
use std::os::raw::c_void;
use std::rc::Rc;
use std::sync::{
    mpsc::{channel, Receiver, Sender, TryRecvError},
    Arc, Mutex,
};
use std::thread;

use vst3_sys::{
    base::{char16, kResultFalse, kResultOk, tresult, FIDString, TBool},
    gui::{IPlugFrame, IPlugView, ViewRect},
    utils::SharedVstPtr,
    VST3,
};

use egui_glow::{
    egui_winit::{egui, winit},
    glow, EguiGlow,
};
use glutin::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    platform::{
        run_return::EventLoopExtRunReturn,
        unix::{EventLoopBuilderExtUnix, WindowBuilderExtUnix},
    },
    window::WindowBuilder,
    PossiblyCurrent, WindowedContext,
};

use crate::vst3::utils;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

struct ParentWindow(*mut c_void);
unsafe impl Send for ParentWindow {}
unsafe impl Sync for ParentWindow {}

enum GUIMessage {
    Terminate,
}

struct GUIThread {
    // SoyBoy specific
    slider: f64,
    // window stuff
    quit: bool,
    needs_repaint: bool,
    // threading stuff
    receiver: Arc<Mutex<Receiver<GUIMessage>>>,
    // egui stuff
    egui_glow: EguiGlow,
    window: WindowedContext<PossiblyCurrent>,
    glow_context: Rc<glow::Context>,
}

// originally from here:
//   https://github.com/emilk/egui/blob/7cd285ecbc2d319f1feac7b9fd9464d06a5ccf77/egui_glow/examples/pure_glow.rs
impl GUIThread {
    fn setup(
        parent: ParentWindow,
        receiver: Arc<Mutex<Receiver<GUIMessage>>>,
    ) -> (Self, EventLoop<()>) {
        let parent_id: usize = if parent.0.is_null() {
            0
        } else {
            parent.0 as usize
        };
        let event_loop = EventLoopBuilder::new().with_any_thread(true).build();

        let window_builder = WindowBuilder::new()
            .with_x11_parent(parent_id.try_into().unwrap())
            .with_resizable(true)
            .with_inner_size(winit::dpi::LogicalSize {
                width: 800.0f32,
                height: 600.0f32,
            })
            .with_title("egui_glow example");

        let window = unsafe {
            glutin::ContextBuilder::new()
                .with_depth_buffer(0)
                .with_srgb(true)
                .with_stencil_buffer(0)
                .with_vsync(true)
                .build_windowed(window_builder, &event_loop)
                .unwrap()
                .make_current()
                .unwrap()
        };

        let glow_context =
            unsafe { glow::Context::from_loader_function(|s| window.get_proc_address(s)) };
        let glow_context = Rc::new(glow_context);

        let egui_glow = EguiGlow::new(window.window(), glow_context.clone());

        let thread = GUIThread {
            slider: 0.0,
            quit: false,
            needs_repaint: false,
            receiver: receiver,
            egui_glow: egui_glow,
            window: window,
            glow_context: glow_context,
        };

        (thread, event_loop)
    }

    fn draw(&mut self) {
        let mut clear_color = [0.1, 0.1, 0.1];

        self.needs_repaint = self.egui_glow.run(self.window.window(), |egui_ctx| {
            egui::SidePanel::left("my_side_panel").show(egui_ctx, |ui| {
                ui.heading("Hello World!");
                if ui.button("Quit").clicked() {}
                ui.color_edit_button_rgb(&mut clear_color);
            });
        });

        // OpenGL drawing
        {
            unsafe {
                use glow::HasContext as _;
                self.glow_context
                    .clear_color(clear_color[0], clear_color[1], clear_color[2], 1.0);
                self.glow_context.clear(glow::COLOR_BUFFER_BIT);
            }

            self.egui_glow.paint(self.window.window());

            // draw things on top of egui here

            self.window.swap_buffers().unwrap();
        }
    }

    fn proc_events(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        match self.receiver.lock().unwrap().try_recv() {
            Ok(message) => match message {
                GUIMessage::Terminate => {
                    println!("try_recv() receive Message::Terminate");
                    self.quit = true;
                }
            },
            Err(err) => match err {
                TryRecvError::Empty => {
                    // println!("try_recv() fails because empty");
                }
                TryRecvError::Disconnected => {
                    println!("try_recv() fails because disconnected");
                    self.quit = true;
                }
            },
        }

        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            Event::RedrawEventsCleared if cfg!(windows) => {
                self.draw();
                if self.needs_repaint {
                    self.window.window().request_redraw();
                    *control_flow = ControlFlow::Poll;
                } else {
                    *control_flow = ControlFlow::Wait;
                }
            }
            Event::RedrawRequested(_) if !cfg!(windows) => {
                self.draw();
                if self.needs_repaint {
                    self.window.window().request_redraw();
                    *control_flow = ControlFlow::Poll;
                } else {
                    *control_flow = ControlFlow::Wait;
                }
            }

            Event::WindowEvent { event, .. } => {
                if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                    self.quit = true;
                }

                if let WindowEvent::Resized(physical_size) = &event {
                    self.window.resize(*physical_size);
                } else if let WindowEvent::ScaleFactorChanged { new_inner_size, .. } = &event {
                    self.window.resize(**new_inner_size);
                }

                self.egui_glow.on_event(&event);
                self.window.window().request_redraw(); // TODO: ask egui if the events warrants a repaint instead
            }
            Event::LoopDestroyed => {
                println!("LoopDestroyed is signaled.");
                self.egui_glow.destroy();
            }

            _ => (),
        }

        if self.quit {
            *control_flow = ControlFlow::Exit;
        }
    }

    fn run_loop(parent: ParentWindow, receiver: Arc<Mutex<Receiver<GUIMessage>>>) {
        let (mut thread, mut event_loop) = GUIThread::setup(parent, receiver);

        event_loop.run_return(move |event, _, control_flow| {
            if !thread.quit {
                thread.draw()
            };
            thread.proc_events(event, control_flow);
        });
    }
}

#[VST3(implements(IPlugView, IPlugFrame))]
pub struct SoyBoyGUI {
    handle: RefCell<Option<thread::JoinHandle<()>>>,
    sender: Arc<Mutex<Sender<GUIMessage>>>,
    receiver: Arc<Mutex<Receiver<GUIMessage>>>,
}

impl SoyBoyGUI {
    pub fn new() -> Box<Self> {
        let handle = RefCell::new(None);
        let (send, resv) = channel();
        let send = Arc::new(Mutex::new(send));
        let recv = Arc::new(Mutex::new(resv));

        SoyBoyGUI::allocate(handle, send, recv)
    }

    fn start_gui(&self, parent: ParentWindow) {
        let (send, resv) = channel();
        let recv = Arc::new(Mutex::new(resv));
        (*self.sender.lock().unwrap()) = send;

        let handle = thread::spawn(move || {
            GUIThread::run_loop(parent, recv);
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

    unsafe fn attached(&self, parent: *mut c_void, type_: FIDString) -> tresult {
        println!("IPlugView::attached()");
        let type_ = utils::fidstring_to_string(type_);

        if type_ == "X11EmbedWindowID" {
            let parent = ParentWindow(parent);
            self.start_gui(parent);
            kResultOk
        } else {
            kResultFalse
        }
    }

    unsafe fn removed(&self) -> tresult {
        println!("IPlugView::removed()");
        let old_handle = self.handle.replace(None);
        let _ = self.sender.lock().unwrap().send(GUIMessage::Terminate);
        println!("sended terminate.");
        let res = old_handle.unwrap().join();
        println!("joined: {:?}", res);
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
