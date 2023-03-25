use egui_extras::image::RetainedImage;
use egui_winit::egui;

const IMG_EDAMAME: &[u8] = include_bytes!("../../resources/edamame.png");
const IMG_VERSION_FRAME: &[u8] = include_bytes!("../../resources/version-frame.png");
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
const IMG_BUTTON_MINUS: &[u8] = include_bytes!("../../resources/button-minus.png");
const IMG_BUTTON_PLUS: &[u8] = include_bytes!("../../resources/button-plus.png");
const IMG_SLIDER_BORDER: &[u8] = include_bytes!("../../resources/slider-border.png");
const IMG_VALUE_ATLAS: &[u8] = include_bytes!("../../resources/paramval.png");
const IMG_PARAM_ATLAS: &[u8] = include_bytes!("../../resources/paramname.png");
const IMG_SELECT_OSC_TYPE: &[u8] = include_bytes!("../../resources/select-osc-type.png");
const IMG_SELECT_OSC_SQ_DUTY: &[u8] = include_bytes!("../../resources/select-osc-square-duty.png");
const IMG_SELECT_SWEEP_TYPE: &[u8] = include_bytes!("../../resources/select-sweep-type.png");
const IMG_SELECT_STUTTER_TIMING: &[u8] =
    include_bytes!("../../resources/select-stutter-timing.png");
const IMG_WAVETABLE_BORDER: &[u8] = include_bytes!("../../resources/wavetable-border.png");
const IMG_OSCILLOSCOPE_BORDER: &[u8] = include_bytes!("../../resources/oscilloscope-border.png");

#[derive(Copy, Clone)]
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

pub struct Images {
    pub edamame: RetainedImage,
    pub version_frame: RetainedImage,
    pub label_logo: RetainedImage,
    pub label_global: RetainedImage,
    pub label_square: RetainedImage,
    pub label_noise: RetainedImage,
    pub label_wavetable: RetainedImage,
    pub label_envelope: RetainedImage,
    pub label_sweep: RetainedImage,
    pub label_stutter: RetainedImage,
    pub button_reset_random: RetainedImage,
    pub button_reset_sine: RetainedImage,
    pub button_minus: RetainedImage,
    pub button_plus: RetainedImage,
    pub slider_border: RetainedImage,
    pub select_osc_type: RetainedImage,
    pub select_osc_sq_duty: RetainedImage,
    pub select_sweep_type: RetainedImage,
    pub select_stutter_timing: RetainedImage,
    pub wavetable_border: RetainedImage,
    pub oscilloscope_border: RetainedImage,
    pub value_atlas: RetainedImage,
    pub param_atlas: RetainedImage,
}

impl Images {
    pub fn new() -> Self {
        Self {
            edamame: RetainedImage::from_image_bytes("soyboy:edamame", IMG_EDAMAME).unwrap(),
            version_frame: RetainedImage::from_image_bytes("soyboy:version", IMG_VERSION_FRAME)
                .unwrap(),
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
            button_minus: RetainedImage::from_image_bytes("soyboy:button:minus", IMG_BUTTON_MINUS)
                .unwrap(),
            button_plus: RetainedImage::from_image_bytes("soyboy:button:plus", IMG_BUTTON_PLUS)
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
            select_stutter_timing: RetainedImage::from_image_bytes(
                "soyboy:select:stutter-timing",
                IMG_SELECT_STUTTER_TIMING,
            )
            .unwrap(),
            wavetable_border: RetainedImage::from_image_bytes(
                "soyboy:wavetable:border",
                IMG_WAVETABLE_BORDER,
            )
            .unwrap(),
            oscilloscope_border: RetainedImage::from_image_bytes(
                "soyboy:oscilloscope:border",
                IMG_OSCILLOSCOPE_BORDER,
            )
            .unwrap(),
            value_atlas: RetainedImage::from_image_bytes("value_atlas", IMG_VALUE_ATLAS).unwrap(),
            param_atlas: RetainedImage::from_image_bytes("name_atlas", IMG_PARAM_ATLAS).unwrap(),
        }
    }
}
