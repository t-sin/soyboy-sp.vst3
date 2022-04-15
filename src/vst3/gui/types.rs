use std::os::raw::c_void;

use egui_extras::image::RetainedImage;
use egui_glow::egui_winit::egui;

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
