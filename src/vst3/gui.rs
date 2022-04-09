use std::cell::RefCell;
use std::os::raw::c_void;
use std::rc::Rc;
use std::sync::{
    mpsc::{channel, Receiver, Sender, TryRecvError},
    Arc, Mutex,
};
use std::thread;
use std::time;

use vst3_sys::{
    base::{char16, kResultFalse, kResultOk, tresult, FIDString, TBool},
    gui::{IPlugFrame, IPlugView, IPlugViewContentScaleSupport, ViewRect},
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

#[derive(Clone)]
struct Label {
    image: Rc<RetainedImage>,
    sense: egui::Sense,
    x: f32,
    y: f32,
}

impl Label {
    fn new(image: Rc<RetainedImage>, x: f32, y: f32) -> Self {
        Self {
            image: image,
            sense: egui::Sense::focusable_noninteractive(),
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

impl egui::widgets::Widget for Label {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let rect = self.rect();

        let response = ui.allocate_rect(rect, self.sense);

        if ui.is_rect_visible(rect) {
            let img = egui::widgets::Image::new(self.image.texture_id(ui.ctx()), rect.size());
            img.paint_at(ui, rect);
        }

        response
    }
}

#[derive(Clone)]
struct Button {
    image: Rc<RetainedImage>,
    sense: egui::Sense,
    clicked_at: time::Instant,
    x: f32,
    y: f32,
}

impl Button {
    fn new(image: Rc<RetainedImage>, x: f32, y: f32) -> Self {
        Self {
            image: image,
            sense: egui::Sense::click().union(egui::Sense::hover()),
            clicked_at: time::Instant::now(),
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

impl egui::widgets::Widget for Button {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let size = self.image.size();
        //        println!("elapsed from clicked is {:?}", self.clicked_at.elapsed());
        let rect = if self.clicked_at.elapsed() < time::Duration::from_millis(500) {
            egui::Rect {
                min: egui::pos2(self.x + 12.0, self.y + 2.0),
                max: egui::pos2(self.x + size[0] as f32 + 2.0, self.y + size[1] as f32 + 2.0),
            }
        } else {
            egui::Rect {
                min: egui::pos2(self.x, self.y),
                max: egui::pos2(self.x + size[0] as f32, self.y + size[1] as f32),
            }
        };

        let response = ui.allocate_rect(rect, self.sense);

        if response.clicked() {
            println!("clicked");
            self.clicked_at = time::Instant::now();
        }

        if ui.is_rect_visible(rect) {
            let img = egui::widgets::Image::new(self.image.texture_id(ui.ctx()), rect.size());
            img.paint_at(ui, rect);

            if response.hovered() {
                ui.painter().rect_filled(
                    rect,
                    egui::Rounding::none(),
                    egui::Color32::from_rgba_unmultiplied(0xab, 0xbb, 0xa8, 100),
                );
            }
        }

        response
    }
}

enum GUIMessage {
    Terminate,
}

struct GUIThread {
    // SoyBoy resources
    label_logo: Label,
    label_global: Label,
    label_square: Label,
    label_noise: Label,
    label_wavetable: Label,
    label_envelope: Label,
    label_sweep: Label,
    label_stutter: Label,
    button_reset_random: Button,
    button_reset_sine: Button,
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
            label_logo: Label::new(
                Rc::new(RetainedImage::from_image_bytes("soyboy:logo", IMG_LOGO).unwrap()),
                6.0,
                6.0,
            ),
            label_global: Label::new(
                Rc::new(
                    RetainedImage::from_image_bytes("soyboy:label:global", IMG_LABEL_GLOBAL)
                        .unwrap(),
                ),
                24.0,
                86.0,
            ),
            label_square: Label::new(
                Rc::new(
                    RetainedImage::from_image_bytes("soyboy:label:square", IMG_LABEL_SQUARE)
                        .unwrap(),
                ),
                24.0,
                216.0,
            ),
            label_noise: Label::new(
                Rc::new(
                    RetainedImage::from_image_bytes("soyboy:label:noise", IMG_LABEL_NOISE).unwrap(),
                ),
                240.0,
                280.0,
            ),
            label_wavetable: Label::new(
                Rc::new(
                    RetainedImage::from_image_bytes("soyboy:label:wavetable", IMG_LABEL_WAVETABLE)
                        .unwrap(),
                ),
                24.0,
                408.0,
            ),
            label_envelope: Label::new(
                Rc::new(
                    RetainedImage::from_image_bytes("soyboy:label:envelope", IMG_LABEL_ENVELOPE)
                        .unwrap(),
                ),
                352.0,
                12.0,
            ),
            label_sweep: Label::new(
                Rc::new(
                    RetainedImage::from_image_bytes("soyboy:label:sweep", IMG_LABEL_SWEEP).unwrap(),
                ),
                352.0,
                184.0,
            ),
            label_stutter: Label::new(
                Rc::new(
                    RetainedImage::from_image_bytes("soyboy:label:stutter", IMG_LABEL_STUTTER)
                        .unwrap(),
                ),
                352.0,
                316.0,
            ),
            button_reset_random: Button::new(
                Rc::new(
                    RetainedImage::from_image_bytes(
                        "soyboy:button:reset-random",
                        IMG_BUTTON_RESET_RANDOM,
                    )
                    .unwrap(),
                ),
                206.0,
                526.0,
            ),
            button_reset_sine: Button::new(
                Rc::new(
                    RetainedImage::from_image_bytes(
                        "soyboy:button:reset-sine",
                        IMG_BUTTON_RESET_SINE,
                    )
                    .unwrap(),
                ),
                274.0,
                526.0,
            ),
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
            let show_label = |name: &str, label: Label| {
                let rect = label.rect();
                egui::Area::new(name)
                    .fixed_pos(rect.min)
                    .interactable(false)
                    .show(egui_ctx, |ui| ui.add(label));
            };
            let show_button = |name: &str, button: Button, do_click: &dyn Fn()| {
                let rect = button.rect();
                egui::Area::new(name)
                    .fixed_pos(rect.min)
                    .movable(false)
                    .show(egui_ctx, |ui| {
                        let resp = ui.add(button);
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
            show_label("logo", self.label_logo.clone());

            // labels
            {
                // left side
                show_label("label: global", self.label_global.clone());
                show_label("label: square", self.label_square.clone());
                show_label("label: noise", self.label_noise.clone());
                show_label("label: wavetable", self.label_wavetable.clone());

                // right side
                show_label("label: envelope", self.label_envelope.clone());
                show_label("label: sweep", self.label_sweep.clone());
                show_label("label: stutter", self.label_stutter.clone());
            }

            // buttons
            show_button(
                "button: reset wavetable random",
                self.button_reset_random.clone(),
                &|| {
                    // TODO: write a code reset plugin's wavetable
                    println!("reset random!!!");
                },
            );
            show_button(
                "button: reset wavetable as sine",
                self.button_reset_sine.clone(),
                &|| {
                    // TODO: write a code reset plugin's wavetable
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
            thread.proc_events(event, control_flow);
        });
    }
}

#[VST3(implements(IPlugView, IPlugFrame, IPlugViewContentScaleSupport))]
pub struct SoyBoyGUI {
    scale_factor: RefCell<f32>,
    handle: RefCell<Option<thread::JoinHandle<()>>>,
    sender: RefCell<Option<Sender<GUIMessage>>>,
}

impl SoyBoyGUI {
    pub fn new() -> Box<Self> {
        let scale_factor = RefCell::new(1.0);
        let handle = RefCell::new(None);
        let sender = RefCell::new(None);

        SoyBoyGUI::allocate(scale_factor, handle, sender)
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

impl IPlugViewContentScaleSupport for SoyBoyGUI {
    unsafe fn set_scale_factor(&self, scale_factor: f32) -> tresult {
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
