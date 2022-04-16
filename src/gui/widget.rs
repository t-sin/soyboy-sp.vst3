use std::rc::Rc;
use std::sync::Arc;
use std::time;

use egui_extras::image::RetainedImage;
use egui_glow::egui_winit::{egui, egui::Widget};
use num;

use crate::soyboy::parameters::{Normalizable, ParameterDef, SoyBoyParameter};

use super::{constants::*, types::*};

fn screen_rect() -> egui::Rect {
    egui::Rect {
        min: egui::pos2(0.0, 0.0),
        max: egui::pos2(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
    }
}

#[derive(Clone, Debug)]
pub struct Toggle {
    value: bool,
    prev_value: bool,
}

impl Toggle {
    fn new(v: bool, prev: bool) -> Self {
        Self {
            value: v,
            prev_value: prev,
        }
    }

    fn val(&self) -> bool {
        self.value
    }

    fn set(&mut self, v: bool) {
        self.prev_value = self.value;
        self.value = v;
    }

    fn toggled(&self) -> bool {
        self.value != self.prev_value
    }
}

pub trait Behavior {
    fn update(&mut self) -> bool;
    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response;
    fn rect(&self) -> egui::Rect;
}

// available characters in resources/paramval.png
enum Character {
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Dot,
    Minus,
}

#[derive(Clone)]
struct Region {
    pos: egui::Pos2,
    size: egui::Vec2,
}

impl Region {
    fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            pos: egui::pos2(x, y),
            size: egui::vec2(w, h),
        }
    }
}

impl Character {
    fn from_char(ch: char) -> Option<Character> {
        match ch {
            '0' => Some(Character::Digit0),
            '1' => Some(Character::Digit1),
            '2' => Some(Character::Digit2),
            '3' => Some(Character::Digit3),
            '4' => Some(Character::Digit4),
            '5' => Some(Character::Digit5),
            '6' => Some(Character::Digit6),
            '7' => Some(Character::Digit7),
            '8' => Some(Character::Digit8),
            '9' => Some(Character::Digit9),
            '.' => Some(Character::Dot),
            '-' => Some(Character::Minus),
            _ => None,
        }
    }

    fn get_region(&self) -> Region {
        match self {
            Character::Digit0 => Region::new(0.0, 0.0, 10.0, 14.0),
            Character::Digit1 => Region::new(14.0, 0.0, 6.0, 14.0),
            Character::Digit2 => Region::new(24.0, 0.0, 10.0, 14.0),
            Character::Digit3 => Region::new(36.0, 0.0, 10.0, 14.0),
            Character::Digit4 => Region::new(48.0, 0.0, 10.0, 14.0),
            Character::Digit5 => Region::new(60.0, 0.0, 10.0, 14.0),
            Character::Digit6 => Region::new(72.0, 0.0, 10.0, 14.0),
            Character::Digit7 => Region::new(84.0, 0.0, 10.0, 14.0),
            Character::Digit8 => Region::new(96.0, 0.0, 10.0, 14.0),
            Character::Digit9 => Region::new(108.0, 0.0, 10.0, 14.0),
            Character::Dot => Region::new(132.0, 0.0, 2.0, 14.0),
            Character::Minus => Region::new(136.0, 0.0, 10.0, 14.0),
        }
    }
}

// available units in resources/paramval.png
#[derive(Clone)]
pub enum ParameterUnit {
    None,
    Decibel,
    Cent,
    MilliSec,
    Sec,
    Percent,
}

impl ParameterUnit {
    fn get_region(&self) -> Option<Region> {
        match self {
            ParameterUnit::None => None,
            ParameterUnit::Decibel => Some(Region::new(0.0, 16.0, 22.0, 14.0)),
            ParameterUnit::Cent => Some(Region::new(30.0, 16.0, 58.0, 14.0)),
            ParameterUnit::MilliSec => Some(Region::new(96.0, 16.0, 22.0, 14.0)),
            ParameterUnit::Sec => Some(Region::new(126.0, 16.0, 10.0, 14.0)),
            ParameterUnit::Percent => Some(Region::new(144.0, 16.0, 10.0, 14.0)),
        }
    }
}

#[derive(Clone)]
pub struct ParameterValue {
    atlas: Rc<RetainedImage>,
    regions: Vec<Region>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl ParameterValue {
    pub fn new(
        value_str: String,
        unit: ParameterUnit,
        atlas: Rc<RetainedImage>,
        x: f32,
        y: f32,
    ) -> Self {
        let (regions, w, h) = ParameterValue::layout(value_str, unit);
        Self {
            atlas,
            regions,
            x,
            y,
            w: w,
            h: h,
        }
    }

    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn rect(&self) -> egui::Rect {
        let top_left = egui::pos2(self.x, self.y);
        egui::Rect {
            min: top_left,
            max: top_left + egui::vec2(self.w, self.h),
        }
    }

    fn layout(value_str: String, unit: ParameterUnit) -> (Vec<Region>, f32, f32) {
        let mut regions = Vec::new();
        let (mut w, mut h) = (0.0, 0.0);

        // println!("layout a value {} formatted as {}", value, s);
        for ch in value_str.chars() {
            match Character::from_char(ch) {
                Some(c) => {
                    let region = c.get_region();
                    w += region.size.x;
                    h = region.size.y;
                    regions.push(region);
                }
                None => {
                    // println!("invalid char in the target: '{}'", ch);
                }
            }
        }

        // for the spacing between characters
        w += (value_str.chars().count() - 1) as f32 * 2.0;

        if let Some(region) = unit.get_region() {
            w += region.size.x;
            h = region.size.y;
            regions.push(region);
            // for the spacing between last char and unit string
            w += 2.0;
        }

        (regions, w, h)
    }
}

impl Widget for ParameterValue {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let rect = egui::Rect {
            min: egui::pos2(self.x, self.y),
            max: egui::pos2(self.x + self.w as f32, self.y + self.w as f32),
        };

        let response = ui.allocate_rect(rect, egui::Sense::focusable_noninteractive());

        if ui.is_rect_visible(rect) {
            let atlas_size = self.atlas.size();
            let atlas_size = egui::vec2(atlas_size[0] as f32, atlas_size[1] as f32);
            let top_left = egui::pos2(self.x, self.y);
            let mut offset = egui::pos2(0.0, 0.0);
            let img = egui::widgets::Image::new(self.atlas.texture_id(ui.ctx()), atlas_size);

            for region in self.regions.iter() {
                let clip_rect = egui::Rect {
                    min: top_left,
                    max: top_left + region.size.into(),
                };
                ui.set_clip_rect(clip_rect.translate(offset.to_vec2()));

                let draw_rect = egui::Rect {
                    min: top_left,
                    max: top_left + atlas_size.into(),
                };

                img.paint_at(
                    ui,
                    draw_rect.translate(offset.to_vec2() - region.pos.to_vec2()),
                );

                offset.x += region.size.x + 2.0;
            }
        }

        response
    }
}

// available parameters in resources/paramval.png
impl SoyBoyParameter {
    fn get_region(&self) -> Option<Region> {
        match self {
            SoyBoyParameter::MasterVolume => Some(Region::new(0.0, 0.0, 66.0, 14.0)),
            SoyBoyParameter::Detune => Some(Region::new(0.0, 16.0, 74.0, 14.0)),
            SoyBoyParameter::OscillatorType => Some(Region::new(0.0, 32.0, 88.0, 14.0)),
            SoyBoyParameter::OscSqDuty => Some(Region::new(0.0, 48.0, 104.0, 14.0)),
            SoyBoyParameter::OscNsInterval => Some(Region::new(0.0, 64.0, 82.0, 14.0)),
            SoyBoyParameter::EgAttack => Some(Region::new(0.0, 80.0, 70.0, 14.0)),
            SoyBoyParameter::EgDecay => Some(Region::new(0.0, 96.0, 58.0, 14.0)),
            SoyBoyParameter::EgSustain => Some(Region::new(0.0, 112.0, 74.0, 14.0)),
            SoyBoyParameter::EgRelease => Some(Region::new(0.0, 128.0, 78.0, 14.0)),
            SoyBoyParameter::SweepType => Some(Region::new(0.0, 144.0, 46.0, 14.0)),
            SoyBoyParameter::SweepAmount => Some(Region::new(0.0, 160.0, 70.0, 14.0)),
            SoyBoyParameter::SweepPeriod => Some(Region::new(0.0, 176.0, 62.0, 14.0)),
            SoyBoyParameter::StutterTime => Some(Region::new(0.0, 192.0, 38.0, 14.0)),
            SoyBoyParameter::StutterDepth => Some(Region::new(0.0, 208.0, 58.0, 14.0)),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct ParameterName {
    param: SoyBoyParameter,
    atlas: Rc<RetainedImage>,
    x: f32,
    y: f32,
}

impl ParameterName {
    pub fn new(param: SoyBoyParameter, atlas: Rc<RetainedImage>, x: f32, y: f32) -> Self {
        Self { param, atlas, x, y }
    }
}

impl Widget for ParameterName {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let topleft = egui::pos2(self.x, self.y);
        let region = self.param.get_region().unwrap();

        let rect = egui::Rect {
            min: topleft,
            max: topleft + egui::vec2(region.size.x, region.size.y),
        };

        let response = ui.allocate_rect(rect, egui::Sense::focusable_noninteractive());

        if ui.is_rect_visible(rect) {
            let atlas_size = self.atlas.size();
            let atlas_size = egui::vec2(atlas_size[0] as f32, atlas_size[1] as f32);
            let img = egui::widgets::Image::new(self.atlas.texture_id(ui.ctx()), atlas_size);

            let clip_rect = egui::Rect {
                min: topleft,
                max: topleft + region.size.into(),
            };
            ui.set_clip_rect(clip_rect);

            let draw_rect = egui::Rect {
                min: topleft,
                max: topleft + atlas_size.into(),
            };
            img.paint_at(ui, draw_rect.translate(-region.pos.to_vec2()));
        }

        response
    }
}

#[derive(Clone)]
pub struct ImageLabel {
    image: Image,
    sense: egui::Sense,
    x: f32,
    y: f32,
}

impl ImageLabel {
    pub fn new(image: Image, x: f32, y: f32) -> Self {
        Self {
            image,
            sense: egui::Sense::focusable_noninteractive(),
            x: x,
            y: y,
        }
    }

    pub fn rect(&self) -> egui::Rect {
        let size = self.image.size;
        egui::Rect {
            min: egui::pos2(self.x, self.y),
            max: egui::pos2(self.x + size[0] as f32, self.y + size[1] as f32),
        }
    }
}

impl Widget for ImageLabel {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let rect = self.rect();

        let response = ui.allocate_rect(rect, self.sense);

        if ui.is_rect_visible(rect) {
            let img = egui::widgets::Image::new(self.image.texture_id, rect.size());
            img.paint_at(ui, rect);
        }

        response
    }
}

#[derive(Clone)]
pub struct Button {
    image: Image,
    sense: egui::Sense,
    clicked: bool,
    rect: egui::Rect,
}

impl Button {
    pub fn new(image: Image, clicked: bool, rect: egui::Rect) -> Self {
        Self {
            image: image,
            sense: egui::Sense::click().union(egui::Sense::hover()),
            clicked: clicked,
            rect: rect,
        }
    }
}

impl Widget for &mut Button {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let rect = if self.clicked {
            self.rect.translate(egui::vec2(2.0, 2.0))
        } else {
            self.rect
        };

        let response = ui.allocate_rect(rect, self.sense);

        if ui.is_rect_visible(rect) {
            let img = egui::widgets::Image::new(self.image.texture_id, rect.size());
            img.paint_at(ui, rect);

            if response.hovered() {
                ui.painter().rect_filled(
                    rect,
                    egui::Rounding::none(),
                    egui::Color32::from_rgba_unmultiplied(0xab, 0xbb, 0xa8, 80),
                );
            }
        }

        response
    }
}

#[derive(Clone)]
pub struct ButtonBehavior {
    image: Image,
    clicked_at: time::Instant,
    clicked: Toggle,
    x: f32,
    y: f32,
}

impl ButtonBehavior {
    pub fn new(image: Image, x: f32, y: f32) -> Self {
        Self {
            image: image,
            clicked_at: time::Instant::now(),
            clicked: Toggle::new(false, false),
            x: x,
            y: y,
        }
    }
}

impl Behavior for ButtonBehavior {
    fn rect(&self) -> egui::Rect {
        let size = self.image.size;
        egui::Rect {
            min: egui::pos2(self.x, self.y),
            max: egui::pos2(self.x + size[0] as f32, self.y + size[1] as f32),
        }
    }

    fn update(&mut self) -> bool {
        if self.clicked_at.elapsed() <= time::Duration::from_millis(100) {
            self.clicked.set(true);
        } else {
            self.clicked.set(false);
        }

        self.clicked.toggled()
    }

    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut widget = Button::new(self.image.clone(), self.clicked.val(), self.rect());
        let response = widget.ui(ui);

        if response.clicked() {
            self.clicked_at = time::Instant::now();
        }

        response
    }
}

pub struct Slider {
    border_img: Image,
    sense: egui::Sense,
    rect: egui::Rect,
    bipolar: bool,
    value: f64,
}

impl Slider {
    pub fn new(border_img: Image, value: f64, bipolar: bool, rect: egui::Rect) -> Self {
        Self {
            border_img: border_img,
            sense: egui::Sense::drag(),
            rect: rect,
            bipolar: bipolar,
            value: value,
        }
    }
}

impl Widget for Slider {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let rect_label = self.rect.clone();
        let _ = ui.allocate_rect(
            rect_label,
            egui::Sense {
                click: false,
                drag: false,
                focusable: false,
            },
        );

        if ui.is_rect_visible(rect_label) {}

        let rect_slider = self.rect.clone().translate(egui::vec2(0.0, 8.0));
        let response = ui.allocate_rect(rect_slider, self.sense);

        if ui.is_rect_visible(rect_slider) {
            let w = self.rect.max.x - 2.0 - self.rect.min.x + 2.0;

            if self.bipolar {
                let color = egui::Color32::from_rgb(0x33, 0x3f, 0x32);
                if self.value >= 0.5 {
                    ui.painter().rect_filled(
                        egui::Rect {
                            min: egui::pos2(0.0, 0.0),
                            max: egui::pos2(w / 2.0 * (self.value as f32 - 0.5) * 2.0, 14.0),
                        }
                        .translate(egui::vec2(self.rect.min.x + w / 2.0, self.rect.min.y)),
                        egui::Rounding::none(),
                        color,
                    );
                } else {
                    let ratio = self.value as f32 * 2.0;
                    ui.painter().rect_filled(
                        egui::Rect {
                            min: egui::pos2(0.0, 0.0),
                            max: egui::pos2(w / 2.0 * (1.0 - ratio), 14.0),
                        }
                        .translate(egui::vec2(
                            self.rect.min.x + w / 2.0 * ratio,
                            self.rect.min.y,
                        )),
                        egui::Rounding::none(),
                        color,
                    );
                }

                // center bar
                ui.painter().rect_filled(
                    egui::Rect {
                        min: egui::pos2(0.0, 0.0),
                        max: egui::pos2(2.0, 10.0),
                    }
                    .translate(egui::vec2(
                        self.rect.min.x + (w / 2.0 - 1.0),
                        self.rect.min.y + 2.0,
                    )),
                    egui::Rounding::none(),
                    egui::Color32::from_rgb(0x33, 0x3f, 0x32),
                );
            } else {
                ui.painter().rect_filled(
                    egui::Rect {
                        min: self.rect.min,
                        max: egui::pos2(self.rect.min.x + w * self.value as f32, self.rect.max.y),
                    },
                    egui::Rounding::none(),
                    egui::Color32::from_rgb(0x33, 0x3f, 0x32),
                );
            }

            let img = egui::widgets::Image::new(self.border_img.texture_id, self.rect.size());
            img.paint_at(ui, self.rect);
        }

        response
    }
}

pub struct SliderBehavior {
    border_img: Image,
    bipolar: bool,
    value: f64,
    x: f32,
    y: f32,
    parameter: SoyBoyParameter,
    event_handler: Arc<dyn EventHandler>,
}

impl SliderBehavior {
    pub fn new(
        border_img: Image,
        value: f64,
        bipolar: bool,
        x: f32,
        y: f32,
        parameter: SoyBoyParameter,
        event_handler: Arc<dyn EventHandler>,
    ) -> Self {
        Self {
            border_img: border_img,
            value: value,
            bipolar: bipolar,
            x: x,
            y: y,
            parameter,
            event_handler,
        }
    }
}

impl Behavior for SliderBehavior {
    fn update(&mut self) -> bool {
        false
    }

    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let widget = Slider::new(
            self.border_img.clone(),
            self.value,
            self.bipolar,
            self.rect(),
        );
        let response = ui.add(widget);

        if response.dragged() {
            let delta_factor = if ui.input().modifiers.shift {
                // It may be wrong this way...
                3000.0
            } else {
                300.0
            };

            let delta_x = response.drag_delta().x;
            let delta_v = delta_x as f64 / delta_factor;
            self.value = num::clamp(self.value + delta_v, 0.0, 1.0);
            self.event_handler
                .change_parameter(self.parameter, self.value);
        }

        response
    }

    fn rect(&self) -> egui::Rect {
        let size = self.border_img.size;
        egui::Rect::from_two_pos(
            egui::pos2(self.x, self.y),
            egui::pos2(self.x + size[0] as f32, self.y + size[1] as f32),
        )
    }
}

//    #[derive(Clone)]
pub struct ParameterSlider {
    slider: SliderBehavior,
    param: SoyBoyParameter,
    param_def: ParameterDef,
    unit: ParameterUnit,
    param_atlas: Rc<RetainedImage>,
    value_atlas: Rc<RetainedImage>,
    x: f32,
    y: f32,
}

impl ParameterSlider {
    pub fn new(
        param: SoyBoyParameter,
        param_def: ParameterDef,
        value: f64,
        bipolar: bool,
        unit: ParameterUnit,
        border_img: Image,
        param_atlas: Rc<RetainedImage>,
        value_atlas: Rc<RetainedImage>,
        x: f32,
        y: f32,
        event_handler: Arc<dyn EventHandler>,
    ) -> Self {
        Self {
            param,
            param_def,
            unit,
            slider: SliderBehavior::new(
                border_img,
                value,
                bipolar,
                x,
                y + 16.0,
                param,
                event_handler,
            ),
            param_atlas,
            value_atlas,
            x,
            y,
        }
    }
}

impl Behavior for ParameterSlider {
    fn update(&mut self) -> bool {
        self.slider.update()
    }

    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let rect = self.rect();

        ui.set_clip_rect(rect);
        ui.add(ParameterName::new(
            self.param.clone(),
            self.param_atlas.clone(),
            self.x,
            self.y,
        ));
        ui.set_clip_rect(rect);

        let mut value = ParameterValue::new(
            self.param_def.format(self.slider.value),
            self.unit.clone(),
            self.value_atlas.clone(),
            0.0,
            0.0,
        );
        let size = egui::vec2(266.0, 30.0);
        let value_rect = value.rect().size();
        value.set_pos(self.x + (size.x - value_rect.x), self.y);
        ui.add(value);

        ui.set_clip_rect(screen_rect());

        self.slider.show(ui)
    }

    fn rect(&self) -> egui::Rect {
        let topleft = egui::pos2(self.x, self.y);
        let size = egui::vec2(266.0, 30.0);
        egui::Rect {
            min: topleft,
            max: topleft + size,
        }
    }
}
