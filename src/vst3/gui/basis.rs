use std::os::raw::c_void;

use egui_extras::image::RetainedImage;
use egui_glow::egui_winit::egui;

// images
pub const IMG_LOGO: &[u8] = include_bytes!("../../../resources/logo.png");
pub const IMG_LABEL_GLOBAL: &[u8] = include_bytes!("../../../resources/label-global.png");
pub const IMG_LABEL_SQUARE: &[u8] = include_bytes!("../../../resources/label-osc-square.png");
pub const IMG_LABEL_NOISE: &[u8] = include_bytes!("../../../resources/label-osc-noise.png");
pub const IMG_LABEL_WAVETABLE: &[u8] = include_bytes!("../../../resources/label-osc-wavetable.png");
pub const IMG_LABEL_ENVELOPE: &[u8] = include_bytes!("../../../resources/label-envelope.png");
pub const IMG_LABEL_SWEEP: &[u8] = include_bytes!("../../../resources/label-sweep.png");
pub const IMG_LABEL_STUTTER: &[u8] = include_bytes!("../../../resources/label-stutter.png");
pub const IMG_BUTTON_RESET_RANDOM: &[u8] =
    include_bytes!("../../../resources/button-reset-random.png");
pub const IMG_BUTTON_RESET_SINE: &[u8] = include_bytes!("../../../resources/button-reset-sine.png");
pub const IMG_SLIDER_BORDER: &[u8] = include_bytes!("../../../resources/slider-border.png");
pub const IMG_VALUE_ATLAS: &[u8] = include_bytes!("../../../resources/paramval.png");
pub const IMG_PARAM_ATLAS: &[u8] = include_bytes!("../../../resources/paramname.png");

pub const SCREEN_WIDTH: u32 = 680;
pub const SCREEN_HEIGHT: u32 = 560;

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
