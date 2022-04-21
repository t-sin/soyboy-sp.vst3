use std::os::raw::c_void;

use egui_extras::image::RetainedImage;
use egui_glow::egui_winit::egui;

use crate::soyboy::parameters::SoyBoyParameter;

pub struct ParentWindow(pub *mut c_void);
unsafe impl Send for ParentWindow {}
unsafe impl Sync for ParentWindow {}

#[derive(Clone)]
pub struct Image {
    pub texture_id: egui::TextureId,
    pub size: egui::Vec2,
}

impl Image {
    pub fn new(egui_ctx: &egui::Context, image: &RetainedImage) -> Self {
        Self {
            texture_id: image.texture_id(egui_ctx),
            size: image.size_vec2(),
        }
    }
}

pub trait EventHandler {
    fn change_parameter(&self, p: SoyBoyParameter, value_normalized: f64);
}

pub enum GUIMessage {
    Terminate,
}

pub enum GUIEvent {
    Redraw,
}

pub trait Behavior {
    fn update(&mut self) -> bool;
    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response;
}

#[derive(Clone, Debug)]
pub struct Toggle {
    value: bool,
    prev_value: bool,
}

impl Toggle {
    pub fn new(v: bool, prev: bool) -> Self {
        Self {
            value: v,
            prev_value: prev,
        }
    }

    pub fn val(&self) -> bool {
        self.value
    }

    pub fn set(&mut self, v: bool) {
        self.prev_value = self.value;
        self.value = v;
    }

    pub fn toggled(&self) -> bool {
        self.value != self.prev_value
    }
}

#[derive(Clone)]
pub struct Region {
    pub pos: egui::Pos2,
    pub size: egui::Vec2,
}

impl Region {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            pos: egui::pos2(x, y),
            size: egui::vec2(w, h),
        }
    }
}
