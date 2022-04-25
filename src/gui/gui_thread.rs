use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::ptr::null_mut;
use std::rc::Rc;
use std::sync::{
    mpsc::{Receiver, TryRecvError},
    Arc, Mutex,
};
use std::time;

use egui_extras::image::RetainedImage;
use egui_glow::{egui_winit::egui, glow, EguiGlow};
#[cfg(target_os = "linux")]
use glutin::platform::unix::{EventLoopBuilderExtUnix, WindowBuilderExtUnix};
#[cfg(target_os = "windows")]
use glutin::platform::windows::{EventLoopBuilderExtWindows, WindowBuilderExtWindows};
use glutin::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
    PossiblyCurrent, WindowedContext,
};

use crate::soyboy::parameters::{ParameterDef, SoyBoyParameter};

use super::{constants::*, types::*, widget::*};

pub struct Images {
    edamame: RetainedImage,
    label_logo: RetainedImage,
    label_global: RetainedImage,
    label_square: RetainedImage,
    label_noise: RetainedImage,
    label_wavetable: RetainedImage,
    label_envelope: RetainedImage,
    label_sweep: RetainedImage,
    label_stutter: RetainedImage,
    button_reset_random: RetainedImage,
    button_reset_sine: RetainedImage,
    slider_border: RetainedImage,
    select_osc_type: RetainedImage,
    select_osc_sq_duty: RetainedImage,
    select_sweep_type: RetainedImage,
    value_atlas: RetainedImage,
    param_atlas: RetainedImage,
}

pub struct UI {
    _images: Images,
    edamame: AnimatedEdamame,
    label_logo: ImageLabel,
    label_global: ImageLabel,
    label_square: ImageLabel,
    label_noise: ImageLabel,
    label_wavetable: ImageLabel,
    label_envelope: ImageLabel,
    label_sweep: ImageLabel,
    label_stutter: ImageLabel,
    button_reset_random: ButtonBehavior,
    button_reset_sine: ButtonBehavior,
    param_volume: ParameterSlider,
    param_detune: ParameterSlider,
    param_interval: ParameterSlider,
    param_attack: ParameterSlider,
    param_decay: ParameterSlider,
    param_sustain: ParameterSlider,
    param_release: ParameterSlider,
    param_amount: ParameterSlider,
    param_period: ParameterSlider,
    param_time: ParameterSlider,
    param_depth: ParameterSlider,
    param_osc_type: ParameterSelector,
    param_osc_sq_duty: ParameterSelector,
    param_sweep_type: ParameterSelector,
}

impl UI {
    fn new(
        egui_ctx: &egui::Context,
        param_defs: HashMap<SoyBoyParameter, ParameterDef>,
        param_values: Arc<Mutex<HashMap<u32, f64>>>,
        event_handler: Arc<dyn EventHandler>,
    ) -> Self {
        let images = Images {
            edamame: RetainedImage::from_image_bytes("soyboy:edamame", IMG_EDAMAME).unwrap(),
            label_logo: RetainedImage::from_image_bytes("soyboy:logo", IMG_LOGO).unwrap(),
            label_global: RetainedImage::from_image_bytes("soyboy:label:global", IMG_LABEL_GLOBAL)
                .unwrap(),
            label_square: RetainedImage::from_image_bytes("soyboy:label:square", IMG_LABEL_SQUARE)
                .unwrap(),
            label_noise: RetainedImage::from_image_bytes("soyboy:label:noise", IMG_LABEL_NOISE)
                .unwrap(),
            label_wavetable: RetainedImage::from_image_bytes(
                "soyboy:label:wavetable",
                IMG_LABEL_WAVETABLE,
            )
            .unwrap(),
            label_envelope: RetainedImage::from_image_bytes(
                "soyboy:label:envelope",
                IMG_LABEL_ENVELOPE,
            )
            .unwrap(),
            label_sweep: RetainedImage::from_image_bytes("soyboy:label:sweep", IMG_LABEL_SWEEP)
                .unwrap(),
            label_stutter: RetainedImage::from_image_bytes(
                "soyboy:label:stutter",
                IMG_LABEL_STUTTER,
            )
            .unwrap(),
            button_reset_random: RetainedImage::from_image_bytes(
                "soyboy:button:reset-random",
                IMG_BUTTON_RESET_RANDOM,
            )
            .unwrap(),
            button_reset_sine: RetainedImage::from_image_bytes(
                "soyboy:button:reset-sine",
                IMG_BUTTON_RESET_SINE,
            )
            .unwrap(),
            slider_border: RetainedImage::from_image_bytes(
                "soyboy:slider:border",
                IMG_SLIDER_BORDER,
            )
            .unwrap(),
            select_osc_type: RetainedImage::from_image_bytes(
                "soyboy:select:osc-type",
                IMG_SELECT_OSC_TYPE,
            )
            .unwrap(),
            select_osc_sq_duty: RetainedImage::from_image_bytes(
                "soyboy:select:osc-square-duty",
                IMG_SELECT_OSC_SQ_DUTY,
            )
            .unwrap(),
            select_sweep_type: RetainedImage::from_image_bytes(
                "soyboy:select:sweep-type",
                IMG_SELECT_SWEEP_TYPE,
            )
            .unwrap(),
            value_atlas: RetainedImage::from_image_bytes("value_atlas", IMG_VALUE_ATLAS).unwrap(),
            param_atlas: RetainedImage::from_image_bytes("name_atlas", IMG_PARAM_ATLAS).unwrap(),
        };

        let param_values = param_values.lock().unwrap();
        Self {
            edamame: AnimatedEdamame::new(Image::new(egui_ctx, &images.edamame), 18.0, 14.0),
            label_logo: ImageLabel::new(Image::new(egui_ctx, &images.label_logo), 6.0, 6.0),
            label_global: ImageLabel::new(Image::new(egui_ctx, &images.label_global), 24.0, 86.0),
            label_square: ImageLabel::new(Image::new(egui_ctx, &images.label_square), 24.0, 216.0),
            label_noise: ImageLabel::new(Image::new(egui_ctx, &images.label_noise), 24.0, 280.0),
            label_wavetable: ImageLabel::new(
                Image::new(egui_ctx, &images.label_wavetable),
                24.0,
                408.0,
            ),
            label_envelope: ImageLabel::new(
                Image::new(egui_ctx, &images.label_envelope),
                352.0,
                12.0,
            ),
            label_sweep: ImageLabel::new(Image::new(egui_ctx, &images.label_sweep), 352.0, 184.0),
            label_stutter: ImageLabel::new(
                Image::new(egui_ctx, &images.label_stutter),
                352.0,
                316.0,
            ),
            button_reset_random: ButtonBehavior::new(
                Image::new(egui_ctx, &images.button_reset_random),
                206.0,
                526.0,
            ),
            button_reset_sine: ButtonBehavior::new(
                Image::new(egui_ctx, &images.button_reset_sine),
                274.0,
                526.0,
            ),
            param_volume: ParameterSlider::new(
                SoyBoyParameter::MasterVolume,
                param_defs
                    .get(&SoyBoyParameter::MasterVolume)
                    .unwrap()
                    .clone(),
                *param_values
                    .get(&(SoyBoyParameter::MasterVolume as u32))
                    .unwrap(),
                false,
                ParameterUnit::Decibel,
                Image::new(egui_ctx, &images.slider_border),
                Image::new(egui_ctx, &images.param_atlas),
                Image::new(egui_ctx, &images.value_atlas),
                60.0,
                86.0 + 2.0,
                event_handler.clone(),
            ),
            param_detune: ParameterSlider::new(
                SoyBoyParameter::Detune,
                param_defs.get(&SoyBoyParameter::Detune).unwrap().clone(),
                *param_values.get(&(SoyBoyParameter::Detune as u32)).unwrap(),
                true,
                ParameterUnit::Cent,
                Image::new(egui_ctx, &images.slider_border),
                Image::new(egui_ctx, &images.param_atlas),
                Image::new(egui_ctx, &images.value_atlas),
                60.0,
                122.0 + 2.0,
                event_handler.clone(),
            ),
            param_interval: ParameterSlider::new(
                SoyBoyParameter::OscNsInterval,
                param_defs
                    .get(&SoyBoyParameter::OscNsInterval)
                    .unwrap()
                    .clone(),
                *param_values
                    .get(&(SoyBoyParameter::OscNsInterval as u32))
                    .unwrap(),
                false,
                ParameterUnit::MilliSec,
                Image::new(egui_ctx, &images.slider_border),
                Image::new(egui_ctx, &images.param_atlas),
                Image::new(egui_ctx, &images.value_atlas),
                60.0,
                292.0 + 2.0,
                event_handler.clone(),
            ),
            param_attack: ParameterSlider::new(
                SoyBoyParameter::EgAttack,
                param_defs.get(&SoyBoyParameter::EgAttack).unwrap().clone(),
                *param_values
                    .get(&(SoyBoyParameter::EgAttack as u32))
                    .unwrap(),
                false,
                ParameterUnit::Sec,
                Image::new(egui_ctx, &images.slider_border),
                Image::new(egui_ctx, &images.param_atlas),
                Image::new(egui_ctx, &images.value_atlas),
                388.0,
                24.0 + 2.0,
                event_handler.clone(),
            ),
            param_decay: ParameterSlider::new(
                SoyBoyParameter::EgDecay,
                param_defs.get(&SoyBoyParameter::EgDecay).unwrap().clone(),
                *param_values
                    .get(&(SoyBoyParameter::EgDecay as u32))
                    .unwrap(),
                false,
                ParameterUnit::Sec,
                Image::new(egui_ctx, &images.slider_border),
                Image::new(egui_ctx, &images.param_atlas),
                Image::new(egui_ctx, &images.value_atlas),
                388.0,
                58.0 + 2.0,
                event_handler.clone(),
            ),
            param_sustain: ParameterSlider::new(
                SoyBoyParameter::EgSustain,
                param_defs.get(&SoyBoyParameter::EgSustain).unwrap().clone(),
                *param_values
                    .get(&(SoyBoyParameter::EgSustain as u32))
                    .unwrap(),
                false,
                ParameterUnit::None,
                Image::new(egui_ctx, &images.slider_border),
                Image::new(egui_ctx, &images.param_atlas),
                Image::new(egui_ctx, &images.value_atlas),
                388.0,
                92.0 + 2.0,
                event_handler.clone(),
            ),
            param_release: ParameterSlider::new(
                SoyBoyParameter::EgRelease,
                param_defs.get(&SoyBoyParameter::EgRelease).unwrap().clone(),
                *param_values
                    .get(&(SoyBoyParameter::EgRelease as u32))
                    .unwrap(),
                false,
                ParameterUnit::Sec,
                Image::new(egui_ctx, &images.slider_border),
                Image::new(egui_ctx, &images.param_atlas),
                Image::new(egui_ctx, &images.value_atlas),
                388.0,
                126.0 + 2.0,
                event_handler.clone(),
            ),
            param_amount: ParameterSlider::new(
                SoyBoyParameter::SweepAmount,
                param_defs
                    .get(&SoyBoyParameter::SweepAmount)
                    .unwrap()
                    .clone(),
                *param_values
                    .get(&(SoyBoyParameter::SweepAmount as u32))
                    .unwrap(),
                false,
                ParameterUnit::None,
                Image::new(egui_ctx, &images.slider_border),
                Image::new(egui_ctx, &images.param_atlas),
                Image::new(egui_ctx, &images.value_atlas),
                388.0,
                232.0,
                event_handler.clone(),
            ),
            param_period: ParameterSlider::new(
                SoyBoyParameter::SweepPeriod,
                param_defs
                    .get(&SoyBoyParameter::SweepPeriod)
                    .unwrap()
                    .clone(),
                *param_values
                    .get(&(SoyBoyParameter::SweepPeriod as u32))
                    .unwrap(),
                false,
                ParameterUnit::None,
                Image::new(egui_ctx, &images.slider_border),
                Image::new(egui_ctx, &images.param_atlas),
                Image::new(egui_ctx, &images.value_atlas),
                388.0,
                268.0,
                event_handler.clone(),
            ),
            param_time: ParameterSlider::new(
                SoyBoyParameter::StutterTime,
                param_defs
                    .get(&SoyBoyParameter::StutterTime)
                    .unwrap()
                    .clone(),
                *param_values
                    .get(&(SoyBoyParameter::StutterTime as u32))
                    .unwrap(),
                false,
                ParameterUnit::Sec,
                Image::new(egui_ctx, &images.slider_border),
                Image::new(egui_ctx, &images.param_atlas),
                Image::new(egui_ctx, &images.value_atlas),
                388.0,
                342.0,
                event_handler.clone(),
            ),
            param_depth: ParameterSlider::new(
                SoyBoyParameter::StutterDepth,
                param_defs
                    .get(&SoyBoyParameter::StutterDepth)
                    .unwrap()
                    .clone(),
                *param_values
                    .get(&(SoyBoyParameter::StutterDepth as u32))
                    .unwrap(),
                false,
                ParameterUnit::Percent,
                Image::new(egui_ctx, &images.slider_border),
                Image::new(egui_ctx, &images.param_atlas),
                Image::new(egui_ctx, &images.value_atlas),
                388.0,
                378.0,
                event_handler.clone(),
            ),
            param_osc_type: ParameterSelector::new(
                SoyBoyParameter::OscillatorType,
                param_defs
                    .get(&SoyBoyParameter::OscillatorType)
                    .unwrap()
                    .clone(),
                *param_values
                    .get(&(SoyBoyParameter::OscillatorType as u32))
                    .unwrap(),
                Image::new(egui_ctx, &images.select_osc_type),
                Image::new(egui_ctx, &images.param_atlas),
                60.0,
                159.0,
                event_handler.clone(),
            ),
            param_osc_sq_duty: ParameterSelector::new(
                SoyBoyParameter::OscSqDuty,
                param_defs.get(&SoyBoyParameter::OscSqDuty).unwrap().clone(),
                *param_values
                    .get(&(SoyBoyParameter::OscSqDuty as u32))
                    .unwrap(),
                Image::new(egui_ctx, &images.select_osc_sq_duty),
                Image::new(egui_ctx, &images.param_atlas),
                60.0,
                220.0,
                event_handler.clone(),
            ),
            param_sweep_type: ParameterSelector::new(
                SoyBoyParameter::SweepType,
                param_defs.get(&SoyBoyParameter::SweepType).unwrap().clone(),
                *param_values
                    .get(&(SoyBoyParameter::SweepType as u32))
                    .unwrap(),
                Image::new(egui_ctx, &images.select_sweep_type),
                Image::new(egui_ctx, &images.param_atlas),
                388.0,
                186.0,
                event_handler.clone(),
            ),
            _images: images,
        }
    }
}

pub struct GUIThread {
    ui: UI,
    // window stuff
    quit: bool,
    needs_redraw: bool,
    // threading stuff
    receiver: Arc<Mutex<Receiver<GUIMessage>>>,
    plugin_event_recv: Receiver<GUIEvent>,
    // egui stuff
    egui_glow: EguiGlow,
    window: WindowedContext<PossiblyCurrent>,
    // glow_context: Rc<glow::Context>,
}

// originally from here:
//   https://github.com/emilk/egui/blob/7cd285ecbc2d319f1feac7b9fd9464d06a5ccf77/egui_glow/examples/pure_glow.rs
impl GUIThread {
    fn setup_event_loop(parent: ParentWindow) -> (EventLoop<GUIEvent>, WindowBuilder) {
        #[cfg(target_os = "linux")]
        {
            let parent_id: usize = if parent.0.is_null() {
                0
            } else {
                parent.0 as usize
            };

            let event_loop = EventLoopBuilder::<GUIEvent>::with_user_event()
                .with_any_thread(true)
                .build();
            let window_builder = WindowBuilder::new().with_x11_parent(parent_id);

            (event_loop, window_builder)
        }

        #[cfg(target_os = "windows")]
        {
            let parent_id = if parent.0.is_null() {
                null_mut()
            } else {
                parent.0
            };

            let event_loop = EventLoopBuilder::<GUIEvent>::with_user_event()
                .with_any_thread(true)
                .build();
            let window_builder = WindowBuilder::new()
                .with_parent_window(parent_id as isize)
                .with_decorations(false)
                .with_resizable(false);

            (event_loop, window_builder)
        }
    }

    fn setup(
        parent: ParentWindow,
        param_defs: HashMap<SoyBoyParameter, ParameterDef>,
        param_values: Arc<Mutex<HashMap<u32, f64>>>,
        event_handler: Arc<dyn EventHandler>,
        receiver: Arc<Mutex<Receiver<GUIMessage>>>,
        plugin_event_recv: Receiver<GUIEvent>,
    ) -> (Self, EventLoop<GUIEvent>) {
        let (event_loop, window_builder) = Self::setup_event_loop(parent);
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
        let egui_glow = EguiGlow::new(window.window(), glow_context);

        let scale_factor = window.window().scale_factor();
        #[cfg(debug_assertions)]
        println!("scale factor = {}", scale_factor);
        egui_glow.egui_ctx.set_pixels_per_point(1.0);

        let thread = GUIThread {
            ui: UI::new(
                &egui_glow.egui_ctx,
                param_defs,
                param_values,
                event_handler.clone(),
            ),
            quit: false,
            needs_redraw: false,
            receiver,
            plugin_event_recv,
            egui_glow,
            window,
            // glow_context: glow_context,
        };

        (thread, event_loop)
    }

    pub fn update(&mut self, proxy: EventLoopProxy<GUIEvent>) {
        let behaviors: &mut [&mut dyn Behavior] = &mut [
            &mut self.ui.edamame as &mut dyn Behavior,
            &mut self.ui.button_reset_random as &mut dyn Behavior,
            &mut self.ui.button_reset_sine as &mut dyn Behavior,
        ];
        self.needs_redraw = false;

        for widget in behaviors.iter_mut() {
            self.needs_redraw |= widget.update();
        }

        if self.needs_redraw {
            let _ = proxy.send_event(GUIEvent::Redraw);
        }

        if let Ok(ref gui_event) = self.plugin_event_recv.try_recv() {
            if gui_event == &GUIEvent::NoteOn {
                self.ui.edamame.jump();
                self.needs_redraw = true;
            }
        }
    }

    pub fn draw(&mut self) {
        self.needs_redraw |= self.egui_glow.run(self.window.window(), |egui_ctx| {
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

            // labels
            let _ = egui::Area::new("labels")
                .fixed_pos(egui::pos2(0.0, 0.0))
                .movable(false)
                .show(egui_ctx, |ui| {
                    // logo
                    let _ = ui.add(self.ui.label_logo.clone());

                    // left side
                    let _ = ui.add(self.ui.label_global.clone());
                    let _ = ui.add(self.ui.label_square.clone());
                    let _ = ui.add(self.ui.label_noise.clone());
                    let _ = ui.add(self.ui.label_wavetable.clone());

                    // right side
                    let _ = ui.add(self.ui.label_envelope.clone());
                    let _ = ui.add(self.ui.label_sweep.clone());
                    let _ = ui.add(self.ui.label_stutter.clone());
                });

            // params
            let _ = egui::Area::new("params")
                .fixed_pos(egui::pos2(0.0, 0.0))
                .movable(false)
                .show(egui_ctx, |ui| {
                    let _ = self.ui.edamame.show(ui);

                    let resp = self.ui.button_reset_random.show(ui);
                    if resp.clicked() {
                        // TODO: write a code reset plugin's wavetable
                        println!("reset random!!!");
                    }

                    let resp = self.ui.button_reset_sine.show(ui);
                    if resp.clicked() {
                        // TODO: write a code reset plugin's wavetable
                        println!("reset sine!!!");
                    }

                    let _ = self.ui.param_volume.show(ui);
                    let _ = self.ui.param_detune.show(ui);
                    let _ = self.ui.param_interval.show(ui);
                    let _ = self.ui.param_attack.show(ui);
                    let _ = self.ui.param_decay.show(ui);
                    let _ = self.ui.param_sustain.show(ui);
                    let _ = self.ui.param_release.show(ui);

                    let _ = self.ui.param_amount.show(ui);
                    let _ = self.ui.param_period.show(ui);

                    let _ = self.ui.param_time.show(ui);
                    let _ = self.ui.param_depth.show(ui);

                    let _ = self.ui.param_osc_type.show(ui);
                    let _ = self.ui.param_osc_sq_duty.show(ui);
                    let _ = self.ui.param_sweep_type.show(ui);
                });
        });

        // OpenGL drawing
        {
            self.egui_glow.paint(self.window.window());

            // draw things on top of egui here

            self.window.swap_buffers().unwrap();
        }
    }

    pub fn proc_events(&mut self, event: Event<GUIEvent>, control_flow: &mut ControlFlow) {
        match self.receiver.lock().unwrap().try_recv() {
            Ok(message) => match message {
                GUIMessage::Terminate => {
                    #[cfg(debug_assertions)]
                    println!("try_recv() receive Message::Terminate");
                    self.quit = true;
                }
            },
            Err(err) => match err {
                TryRecvError::Empty => {
                    // #[cfg(debug_assertions)]
                    // println!("try_recv() fails because empty");
                }
                TryRecvError::Disconnected => {
                    #[cfg(debug_assertions)]
                    println!("try_recv() fails because disconnected");
                    self.quit = true;
                }
            },
        }

        let mut redraw = || {
            self.draw();
            if self.needs_redraw {
                self.window.window().request_redraw();
            }
        };

        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            Event::RedrawEventsCleared if cfg!(windows) => redraw(),
            Event::RedrawRequested(_) if !cfg!(windows) => redraw(),
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::Destroyed => println!("WindowEvent::Destroyed received, but it may be re-opened GUI so ignore it."),
                    WindowEvent::CloseRequested => self.quit = true,
                    _ => (),
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
            Event::UserEvent(gui_event) => match gui_event {
                GUIEvent::Redraw => redraw(),
                _ => (),
            },
            _ => (),
        }

        if self.quit {
            *control_flow = ControlFlow::Exit;
        } else if self.needs_redraw {
            *control_flow = ControlFlow::Poll;
        } else {
            *control_flow =
                ControlFlow::WaitUntil(time::Instant::now() + time::Duration::from_millis(100));
        }
    }

    pub fn run_loop(
        parent: ParentWindow,
        param_defs: HashMap<SoyBoyParameter, ParameterDef>,
        param_values: Arc<Mutex<HashMap<u32, f64>>>,
        event_handler: Arc<dyn EventHandler>,
        receiver: Arc<Mutex<Receiver<GUIMessage>>>,
        plugin_event_recv: Receiver<GUIEvent>,
    ) {
        let (mut thread, mut event_loop) = GUIThread::setup(
            parent,
            param_defs,
            param_values,
            event_handler,
            receiver,
            plugin_event_recv,
        );
        let proxy = event_loop.create_proxy();

        event_loop.run_return(move |event, _, control_flow| {
            thread.update(proxy.clone());
            thread.proc_events(event, control_flow);
        });
    }
}
