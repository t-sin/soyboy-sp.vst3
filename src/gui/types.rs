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
    fn tell_parameter_changes(&self, p: SoyBoyParameter, value_normalized: f64);
}

pub enum GUIMessage {
    Terminate,
}

pub enum GUIEvent {
    Redraw,
}
