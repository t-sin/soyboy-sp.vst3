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

use egui_glow::{egui_winit::egui, glow, EguiGlow};
#[cfg(target_os = "linux")]
use glutin::platform::unix::{EventLoopBuilderExtUnix, WindowBuilderExtUnix};
#[cfg(target_os = "windows")]
use glutin::platform::windows::{EventLoopBuilderExtWindows, WindowBuilderExtWindows};
use glutin::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
    PossiblyCurrent, WindowedContext,
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
    window: WindowedContext<PossiblyCurrent>,
    // glow_context: Rc<glow::Context>,
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
        receiver: Arc<Mutex<Receiver<GUIThreadMessage>>>,
        plugin_event_recv: Receiver<GUIEvent>,
        controller_connection: Arc<Mutex<ControllerConnection>>,
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
