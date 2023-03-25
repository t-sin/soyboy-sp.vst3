use std::cell::RefCell;
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::ptr::null_mut;
use std::rc::Rc;
use std::sync::{
    mpsc::{Receiver, TryRecvError},
    Arc, Mutex,
};
use std::time;

use cty::c_ulong;

use egui_glow::{glow, EguiGlow};
use egui_winit::egui;
use glutin::{
    config::{Config, ConfigSurfaceTypes, ConfigTemplate, ConfigTemplateBuilder, GlConfig},
    context::{ContextApi, ContextAttributesBuilder},
    display::{Display, GlDisplay},
    surface::{Surface, SurfaceAttributes, SurfaceAttributesBuilder, SwapInterval, WindowSurface},
};
#[cfg(target_os = "linux")]
use raw_window_handle::XlibWindowHandle;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawWindowHandle};
#[cfg(target_os = "windows")]
use winit::platform::windows::{EventLoopBuilderExtWindows, WindowBuilderExtWindows};
#[cfg(target_os = "linux")]
use winit::platform::x11::EventLoopBuilderExtX11;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    platform::run_return::EventLoopExtRunReturn,
    window::{Window, WindowBuilder},
};

use crate::common::{constants, GUIEvent, GUIThreadMessage, Vst3Message};
use crate::soyboy::parameters::{ParameterDef, SoyBoyParameter};
use crate::vst3::ControllerConnection;

use super::{types::*, ui::UI};

pub struct GUIThread {
    ui: UI,
    // window stuff
    quit: bool,
    last_redrawed_at: RefCell<Option<time::Instant>>,
    needs_redraw: bool,
    waveform_view_enabled: Rc<RefCell<bool>>,
    // threading stuff
    receiver: Arc<Mutex<Receiver<GUIThreadMessage>>>,
    plugin_event_recv: Receiver<GUIEvent>,
    controller_connection: Arc<Mutex<ControllerConnection>>,
    // egui stuff
    egui_glow: EguiGlow,
    window: (Window, Surface<WindowSurface>),
}

impl Drop for GUIThread {
    fn drop(&mut self) {
        self.controller_connection
            .lock()
            .unwrap()
            .send_message(Vst3Message::DisableWaveform);
    }
}

// originally from here:
//   https://github.com/emilk/egui/blob/7cd285ecbc2d319f1feac7b9fd9464d06a5ccf77/egui_glow/examples/pure_glow.rs
impl GUIThread {
    fn setup_display(parent: ParentWindow) -> Display {
        #[cfg(target_os = "linux")]
        let (event_loop, window) = {
            let parent_id: c_ulong = if parent.0.is_null() {
                0
            } else {
                parent.0 as c_ulong
            };
            let mut parent_window = XlibWindowHandle::empty();
            parent_window.window = parent_id;
            let parent_window = RawWindowHandle::Xlib(parent_window);

            let event_loop = EventLoopBuilder::<GUIEvent>::with_user_event()
                .with_any_thread(true)
                .build();
            let window = WindowBuilder::new()
                .with_parent_window(Some(parent_window))
                .build(&event_loop)
                .unwrap();

            (event_loop, window)
        };

        #[cfg(target_os = "windows")]
        let (event_loop, window) = {
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
        };

        let raw_display = event_loop.raw_display_handle();
        let raw_window_handle = window.raw_window_handle();

        #[cfg(egl_backend)]
        let preference = DisplayApiPreference::Egl;
        #[cfg(glx_backend)]
        let preference = DisplayApiPreference::Glx(Box::new(unix::register_xlib_error_hook));
        #[cfg(cgl_backend)]
        let preference = DisplayApiPreference::Cgl;
        #[cfg(wgl_backend)]
        let preference = DisplayApiPreference::Wgl(Some(raw_window_handle.unwrap()));
        #[cfg(all(egl_backend, wgl_backend))]
        let preference = DisplayApiPreference::WglThenEgl(Some(raw_window_handle.unwrap()));
        #[cfg(all(egl_backend, glx_backend))]
        let preference = DisplayApiPreference::GlxThenEgl(Box::new(unix::register_xlib_error_hook));
        let gl_display = unsafe { Display::new(raw_display, preference).unwrap() };
        println!("Running on: {}", gl_display.version_string());

        let mut builder = ConfigTemplateBuilder::new().with_alpha_size(8);
        builder = builder
            .compatible_with_native_window(raw_window_handle)
            .with_surface_type(ConfigSurfaceTypes::WINDOW);
        #[cfg(cgl_backend)]
        let builder = builder.with_transparency(true).with_multisampling(8);
        let template = builder.build();

        let config = unsafe { gl_display.find_configs(template) }
            .unwrap()
            .reduce(|acc, config| {
                if config.num_samples() > acc.num_samples() {
                    config
                } else {
                    acc
                }
            })
            .unwrap();
        println!("Picked a config with {} samples", config.num_samples());

        let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));
        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(Some(raw_window_handle));
        let mut not_current_gl_context = Some(unsafe {
            gl_display
                .create_context(&config, &context_attributes)
                .unwrap_or_else(|_| {
                    gl_display
                        .create_context(&config, &fallback_context_attributes)
                        .expect("failed to create context")
                })
        });

        gl_display
    }

    fn setup(
        parent: ParentWindow,
        param_defs: HashMap<SoyBoyParameter, ParameterDef>,
        param_values: Arc<Mutex<HashMap<u32, f64>>>,
        event_handler: Arc<dyn EventHandler>,
        receiver: Arc<Mutex<Receiver<GUIThreadMessage>>>,
        plugin_event_recv: Receiver<GUIEvent>,
        controller_connection: Arc<Mutex<ControllerConnection>>,
    ) -> (Self, EventLoop<GUIEvent>) {
        let display = Self::setup_display(parent);

        let glow_context =
            unsafe { glow::Context::from_loader_function(|s| window.get_proc_address(s)) };
        let glow_context = Rc::new(glow_context);
        let egui_glow = EguiGlow::new(window.window(), glow_context);

        // let scale_factor = window.window().scale_factor();
        // #[cfg(debug_assertions)]
        // println!("scale factor = {}", scale_factor);
        egui_glow.egui_ctx.set_pixels_per_point(1.0);

        let waveform_view_enabled = Rc::new(RefCell::new(false));

        let thread = GUIThread {
            ui: UI::new(
                &egui_glow.egui_ctx,
                param_defs,
                param_values,
                event_handler.clone(),
                controller_connection.clone(),
                waveform_view_enabled.clone(),
            ),
            quit: false,
            last_redrawed_at: RefCell::new(None),
            needs_redraw: false,
            waveform_view_enabled,
            receiver,
            plugin_event_recv,
            controller_connection,
            egui_glow,
            window,
            // glow_context: glow_context,
        };

        (thread, event_loop)
    }

    fn draw_ui(&mut self) {
        self.needs_redraw |= self.egui_glow.run(self.window.window(), |egui_ctx| {
            // background
            egui::Area::new("background").show(egui_ctx, |ui| {
                ui.painter().rect_filled(
                    egui::Rect {
                        min: egui::pos2(0.0, 0.0),
                        max: egui::pos2(
                            constants::SCREEN_WIDTH as f32,
                            constants::SCREEN_HEIGHT as f32,
                        ),
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
                    let _ = ui.add(self.ui.version.clone());

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
                        self.controller_connection
                            .lock()
                            .unwrap()
                            .send_message(Vst3Message::RandomizeWaveTable);
                    }

                    let resp = self.ui.button_reset_sine.show(ui);
                    if resp.clicked() {
                        self.controller_connection
                            .lock()
                            .unwrap()
                            .send_message(Vst3Message::InitializeWaveTable);
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
                    let _ = self.ui.param_stutter_timing.show(ui);

                    let _ = self.ui.param_voices.show(ui);

                    let _ = self.ui.param_wavetable.show(ui);
                    let _ = self.ui.oscilloscope.show(ui);
                });
        });
    }

    pub fn draw(&mut self) {
        self.draw_ui();

        // OpenGL drawing
        let last_redrawed_at = *self.last_redrawed_at.borrow_mut();
        let dur = time::Duration::from_millis(constants::GL_SWAP_INTERVAL_IN_MILLIS);

        if last_redrawed_at.is_none() || last_redrawed_at.unwrap().elapsed() > dur {
            let _ = self.last_redrawed_at.replace(Some(time::Instant::now()));

            self.egui_glow.paint(self.window.window());

            // draw things on top of egui here

            self.window.swap_buffers().unwrap();
        }
    }

    pub fn update(&mut self) {
        let behaviors: &mut [&mut dyn Behavior] = &mut [
            &mut self.ui.edamame as &mut dyn Behavior,
            &mut self.ui.button_reset_random as &mut dyn Behavior,
            &mut self.ui.button_reset_sine as &mut dyn Behavior,
            &mut self.ui.param_voices as &mut dyn Behavior,
        ];

        for widget in behaviors.iter_mut() {
            self.needs_redraw |= widget.update();
        }

        for gui_event in self.plugin_event_recv.try_iter() {
            match gui_event {
                GUIEvent::NoteOn => {
                    self.ui.edamame.jump();
                    self.needs_redraw = true;
                }
                GUIEvent::WaveTableData(table) => {
                    self.ui.param_wavetable.set_wavetable(&table);
                }
                GUIEvent::WaveformData(wf) => {
                    if *self.waveform_view_enabled.borrow() {
                        self.ui.oscilloscope.set_signals(wf.get_signals());
                        self.needs_redraw = true;
                    }
                }
                GUIEvent::Configure(config) => {
                    self.ui.configure(config);
                    self.needs_redraw = true;
                }
                GUIEvent::SetParam(ref param, v) => {
                    self.ui.set_value(param, v);
                    self.needs_redraw = true;
                }
            }
        }

        if self.needs_redraw {
            self.window.window().request_redraw();
        }
    }

    pub fn proc_events(&mut self, event: Event<GUIEvent>, control_flow: &mut ControlFlow) {
        {
            let recv = self.receiver.lock().unwrap();
            for message in recv.try_iter() {
                match message {
                    GUIThreadMessage::Terminate => {
                        self.quit = true;
                    }
                }
            }

            if let Err(TryRecvError::Disconnected) = recv.try_recv() {
                #[cfg(debug_assertions)]
                log::error!("try_recv() fails because disconnected");
                self.quit = true;
            }
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
                    WindowEvent::Destroyed => log::info!("WindowEvent::Destroyed received, but it may be re-opened GUI so ignore it."),
                    WindowEvent::CloseRequested => self.quit = true,
                    _ => (),
                }

                self.egui_glow.on_event(&event);
                self.window.window().request_redraw();
            }
            Event::LoopDestroyed => {
                log::info!("LoopDestroyed is signaled.");
                self.egui_glow.destroy();
            }
            _ => (),
        }

        if self.quit {
            *control_flow = ControlFlow::Exit;
        } else {
            let dur = if *self.waveform_view_enabled.borrow() {
                time::Duration::from_millis(constants::WAVEFORM_UPDATE_INTERVAL_IN_MILLIS)
            } else {
                time::Duration::from_millis(constants::NORMAL_REDRAW_INTERVAL_IN_MILLIS)
            };

            *control_flow = ControlFlow::WaitUntil(time::Instant::now() + dur);
        }
    }

    pub fn run_loop(
        parent: ParentWindow,
        param_defs: HashMap<SoyBoyParameter, ParameterDef>,
        param_values: Arc<Mutex<HashMap<u32, f64>>>,
        event_handler: Arc<dyn EventHandler>,
        receiver: Arc<Mutex<Receiver<GUIThreadMessage>>>,
        plugin_event_recv: Receiver<GUIEvent>,
        controller_connection: Arc<Mutex<ControllerConnection>>,
    ) {
        let (mut thread, mut event_loop) = GUIThread::setup(
            parent,
            param_defs,
            param_values,
            event_handler,
            receiver,
            plugin_event_recv,
            controller_connection,
        );

        event_loop.run_return(move |event, _, control_flow| {
            thread.needs_redraw = false;
            thread.update();
            thread.proc_events(event, control_flow);
        });
    }
}
