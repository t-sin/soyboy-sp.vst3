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

use egui_extras::image::RetainedImage;
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

const SCREEN_WIDTH: u32 = 680;
const SCREEN_HEIGHT: u32 = 560;

// images
const IMG_LOGO: &[u8] = include_bytes!("../../resources/logo.png");
const IMG_LABEL_GLOBAL: &[u8] = include_bytes!("../../resources/label-global.png");
const IMG_LABEL_SQUARE: &[u8] = include_bytes!("../../resources/label-osc-square.png");
const IMG_LABEL_NOISE: &[u8] = include_bytes!("../../resources/label-osc-noise.png");
const IMG_LABEL_WAVETABLE: &[u8] = include_bytes!("../../resources/label-osc-wavetable.png");
const IMG_LABEL_ENVELOPE: &[u8] = include_bytes!("../../resources/label-envelope.png");
const IMG_LABEL_SWEEP: &[u8] = include_bytes!("../../resources/label-sweep.png");
const IMG_LABEL_STUTTER: &[u8] = include_bytes!("../../resources/label-stutter.png");
const IMG_BUTTON_RESET_RANDOM: &[u8] = include_bytes!("../../resources/button-reset-random.png");
const IMG_BUTTON_RESET_SINE: &[u8] = include_bytes!("../../resources/button-reset-sine.png");

struct ParentWindow(*mut c_void);
unsafe impl Send for ParentWindow {}
unsafe impl Sync for ParentWindow {}

struct Button<'a> {
    image: &'a RetainedImage,
    sense: egui::Sense,
    x: f32,
    y: f32,
}

impl<'a> Button<'a> {
    fn new(image: &'a RetainedImage, x: f32, y: f32) -> Self {
        Self {
            image: image,
            sense: egui::Sense {
                click: true,
                drag: false,
                focusable: false,
            },
            x: x,
            y: y,
        }
    }

    fn rect(&self) -> egui::Rect {
        let size = self.image.size();
        egui::Rect {
            min: egui::pos2(self.x, self.y),
            max: egui::pos2(self.x + size[0] as f32, self.y + size[1] as f32),
        }
    }
}

impl<'a> egui::widgets::Widget for Button<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let size = self.image.size();
        let rect = egui::Rect {
            min: egui::pos2(self.x, self.y),
            max: egui::pos2(self.x + size[0] as f32, self.y + size[1] as f32),
        };

        let response = ui.allocate_rect(rect, self.sense);
        if ui.is_rect_visible(rect) {
            let img = egui::widgets::Image::new(self.image.texture_id(ui.ctx()), rect.size());
            img.paint_at(ui, rect);
        }

        response
    }
}

enum GUIMessage {
    Terminate,
}

struct GUIThread {
    // SoyBoy resources
    img_logo: RetainedImage,
    img_label_global: RetainedImage,
    img_label_square: RetainedImage,
    img_label_noise: RetainedImage,
    img_label_wavetable: RetainedImage,
    img_label_envelope: RetainedImage,
    img_label_sweep: RetainedImage,
    img_label_stutter: RetainedImage,
    img_button_reset_random: RetainedImage,
    img_button_reset_sine: RetainedImage,
    // SoyBoy states
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
            .with_resizable(false)
            .with_inner_size(winit::dpi::LogicalSize {
                width: SCREEN_WIDTH as f32,
                height: SCREEN_HEIGHT as f32,
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
            img_logo: RetainedImage::from_image_bytes("soyboy:logo", IMG_LOGO).unwrap(),
            img_label_global: RetainedImage::from_image_bytes(
                "soyboy:label:global",
                IMG_LABEL_GLOBAL,
            )
            .unwrap(),
            img_label_square: RetainedImage::from_image_bytes(
                "soyboy:label:square",
                IMG_LABEL_SQUARE,
            )
            .unwrap(),
            img_label_noise: RetainedImage::from_image_bytes("soyboy:label:noise", IMG_LABEL_NOISE)
                .unwrap(),
            img_label_wavetable: RetainedImage::from_image_bytes(
                "soyboy:label:wavetable",
                IMG_LABEL_WAVETABLE,
            )
            .unwrap(),
            img_label_envelope: RetainedImage::from_image_bytes(
                "soyboy:label:envelope",
                IMG_LABEL_ENVELOPE,
            )
            .unwrap(),
            img_label_sweep: RetainedImage::from_image_bytes("soyboy:label:sweep", IMG_LABEL_SWEEP)
                .unwrap(),
            img_label_stutter: RetainedImage::from_image_bytes(
                "soyboy:label:stutter",
                IMG_LABEL_STUTTER,
            )
            .unwrap(),
            img_button_reset_random: RetainedImage::from_image_bytes(
                "soyboy:button:reset-random",
                IMG_BUTTON_RESET_RANDOM,
            )
            .unwrap(),
            img_button_reset_sine: RetainedImage::from_image_bytes(
                "soyboy:button:reset-sine",
                IMG_BUTTON_RESET_SINE,
            )
            .unwrap(),
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
        // println!(
        //     "cursor pos = {:?}",
        //     self.egui_glow.egui_ctx.input().pointer.interact_pos()
        // );

        self.needs_repaint = self.egui_glow.run(self.window.window(), |egui_ctx| {
            let show_img = |name: &str, img: &RetainedImage, x: f32, y: f32| {
                egui::Area::new(name)
                    .fixed_pos(egui::pos2(x, y))
                    .interactable(false)
                    .show(egui_ctx, |ui| {
                        img.show(ui);
                    });
            };
            let show_button = |name: &str, button: Button, do_click: &dyn Fn()| {
                let rect = button.rect();
                egui::Area::new(name)
                    .fixed_pos(rect.min)
                    .movable(false)
                    .show(egui_ctx, |ui| {
                        let resp = ui.add(button);
                        if resp.hovered() {
                            ui.painter().rect_filled(
                                rect,
                                egui::Rounding::none(),
                                egui::Color32::from_rgba_unmultiplied(0xab, 0xbb, 0xa8, 100),
                            );
                        }
                        if resp.clicked() {
                            do_click();
                        };
                    });
            };

            // background
            egui::Area::new("background").show(egui_ctx, |ui| {
                ui.painter().rect_filled(
                    egui::Rect {
                        min: egui::pos2(0.0, 0.0),
                        max: egui::pos2(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
                    },
                    egui::Rounding::none(),
                    egui::Color32::from_rgb(0xab, 0xbb, 0xa8),
                );
            });

            // logo
            show_img("logo", &self.img_logo, 6.0, 6.0);

            // labels
            {
                // left side
                show_img("label: global", &self.img_label_global, 24.0, 86.0);
                show_img("label: square", &self.img_label_square, 24.0, 216.0);
                show_img("label: noise", &self.img_label_noise, 24.0, 280.0);
                show_img("label: wavetable", &self.img_label_wavetable, 24.0, 408.0);

                // right side
                show_img("label: envelope", &self.img_label_envelope, 352.0, 12.0);
                show_img("label: sweep", &self.img_label_sweep, 352.0, 184.0);
                show_img("label: stutter", &self.img_label_stutter, 352.0, 316.0);
            }

            // buttons
            show_button(
                "button: reset wavetable random",
                Button::new(&self.img_button_reset_random, 206.0, 526.0),
                &|| {
                    println!("reset random!!!");
                },
            );
            show_button(
                "button: reset wavetable as sine",
                Button::new(&self.img_button_reset_sine, 274.0, 526.0),
                &|| {
                    println!("reset sine!!!");
                },
            );
        });

        // OpenGL drawing
        {
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

                // if let WindowEvent::Resized(physical_size) = &event {
                //     self.window.resize(*physical_size);
                // } else if let WindowEvent::ScaleFactorChanged { new_inner_size, .. } = &event {
                //     self.window.resize(**new_inner_size);
                // }

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
    sender: RefCell<Option<Sender<GUIMessage>>>,
}

impl SoyBoyGUI {
    pub fn new() -> Box<Self> {
        let handle = RefCell::new(None);
        let sender = RefCell::new(None);

        SoyBoyGUI::allocate(handle, sender)
    }

    fn start_gui(&self, parent: ParentWindow) {
        let (send, resv) = channel();
        let recv = Arc::new(Mutex::new(resv));
        (*self.sender.borrow_mut()) = Some(send);

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
        let _ = (*self.sender.borrow())
            .as_ref()
            .unwrap()
            .send(GUIMessage::Terminate);
        println!("sended terminate.");
        let res = old_handle.unwrap().join();
        println!("joined: {:?}", res);
        let _ = self.sender.replace(None);
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
        kResultFalse
    }
    unsafe fn check_size_constraint(&self, _rect: *mut ViewRect) -> tresult {
        println!("IPlugView::check_size_constraint()");
        kResultOk
    }
}
