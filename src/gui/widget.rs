use std::sync::Arc;
use std::time;

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

#[derive(Clone)]
pub struct ParameterValue {
    atlas: Image,
    regions: Vec<Region>,
    pos: egui::Pos2,
    size: egui::Vec2,
}

impl ParameterValue {
    pub fn new(value_str: String, unit: ParameterUnit, atlas: Image, x: f32, y: f32) -> Self {
        let (regions, w, h) = ParameterValue::layout(value_str, unit);
        Self {
            atlas,
            regions,
            pos: egui::pos2(x, y),
            size: egui::vec2(w, h),
        }
    }

    pub fn set_pos(&mut self, pos: egui::Pos2) {
        self.pos = pos;
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
        let rect = egui::Rect::from_two_pos(self.pos, self.pos + self.size);

        let response = ui.allocate_rect(rect, egui::Sense::focusable_noninteractive());

        if ui.is_rect_visible(rect) {
            let mut offset = egui::pos2(0.0, 0.0);
            let img = egui::widgets::Image::new(self.atlas.texture_id, self.atlas.size);

            for region in self.regions.iter() {
                let clip_rect = egui::Rect::from_two_pos(self.pos, self.pos + region.size.into());
                ui.set_clip_rect(clip_rect.translate(offset.to_vec2()));

                let draw_rect = egui::Rect::from_two_pos(self.pos, self.pos + self.atlas.size);

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

#[derive(Clone)]
pub struct ParameterName {
    param: SoyBoyParameter,
    atlas: Image,
    pos: egui::Pos2,
}

impl ParameterName {
    pub fn new(param: SoyBoyParameter, atlas: Image, pos: egui::Pos2) -> Self {
        Self { param, atlas, pos }
    }
}

impl Widget for ParameterName {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let region = self.param.get_region().unwrap();

        let rect = egui::Rect::from_two_pos(
            self.pos,
            self.pos + egui::vec2(region.size.x, region.size.y),
        );
        ui.set_clip_rect(rect);

        let response = ui.allocate_rect(rect, egui::Sense::focusable_noninteractive());

        if ui.is_rect_visible(rect) {
            let img = egui::widgets::Image::new(self.atlas.texture_id, self.atlas.size);

            let draw_rect = egui::Rect::from_two_pos(self.pos, self.pos + self.atlas.size);
            img.paint_at(ui, draw_rect.translate(-region.pos.to_vec2()));
        }

        response
    }
}

#[derive(Clone)]
pub struct ImageLabel {
    image: Image,
    sense: egui::Sense,
    pos: egui::Pos2,
}

impl ImageLabel {
    pub fn new(image: Image, x: f32, y: f32) -> Self {
        Self {
            image,
            sense: egui::Sense::focusable_noninteractive(),
            pos: egui::pos2(x, y),
        }
    }
}

impl Widget for ImageLabel {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let rect = egui::Rect::from_two_pos(self.pos, self.pos + self.image.size);

        let response = ui.allocate_rect(rect, self.sense);

        if ui.is_rect_visible(rect) {
            let img = egui::widgets::Image::new(self.image.texture_id, rect.size());
            img.paint_at(ui, rect);
        }

        response
    }
}

#[derive(Clone)]
pub struct Edamame {
    image: Image,
    jumping: bool,
    sense: egui::Sense,
    pos: egui::Pos2,
}

impl Edamame {
    pub fn new(image: Image, jumping: bool, pos: egui::Pos2) -> Self {
        Self {
            image,
            jumping,
            sense: egui::Sense::click(),
            pos,
        }
    }
}

impl Widget for Edamame {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let half_x = self.image.size.x / 2.0;
        let rect = egui::Rect::from_two_pos(
            self.pos,
            self.pos + egui::vec2(self.image.size.x, self.image.size.y),
        );
        let clip_rect =
            egui::Rect::from_two_pos(self.pos, self.pos + egui::vec2(half_x, self.image.size.y));

        if ui.is_rect_visible(clip_rect) {
            let img = egui::widgets::Image::new(self.image.texture_id, rect.size());

            ui.set_clip_rect(clip_rect);

            if self.jumping {
                img.paint_at(ui, rect.translate(egui::vec2(-half_x, 0.0)));
            } else {
                img.paint_at(ui, rect);
            }

            ui.set_clip_rect(screen_rect());
        }

        ui.allocate_rect(clip_rect, self.sense)
    }
}

#[derive(Clone)]
pub struct AnimatedEdamame {
    image: Image,
    jumped_at: time::Instant,
    jumping: Toggle,
    pos: egui::Pos2,
}

impl AnimatedEdamame {
    pub fn new(image: Image, x: f32, y: f32) -> Self {
        Self {
            image,
            jumping: Toggle::new(false, false),
            jumped_at: time::Instant::now(),
            pos: egui::pos2(x, y),
        }
    }
}

impl Behavior for AnimatedEdamame {
    fn update(&mut self) -> bool {
        if self.jumped_at.elapsed() <= time::Duration::from_millis(100) {
            self.jumping.set(true);
        } else {
            self.jumping.set(false);
        }

        self.jumping.toggled()
    }

    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let edamame = Edamame::new(self.image.clone(), self.jumping.val(), self.pos);
        let response = ui.add(edamame);

        if response.clicked() {
            self.jumped_at = time::Instant::now();
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
    pos: egui::Pos2,
}

impl ButtonBehavior {
    pub fn new(image: Image, x: f32, y: f32) -> Self {
        Self {
            image: image,
            clicked_at: time::Instant::now(),
            clicked: Toggle::new(false, false),
            pos: egui::pos2(x, y),
        }
    }
}

impl Behavior for ButtonBehavior {
    fn update(&mut self) -> bool {
        if self.clicked_at.elapsed() <= time::Duration::from_millis(100) {
            self.clicked.set(true);
        } else {
            self.clicked.set(false);
        }

        self.clicked.toggled()
    }

    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let rect = egui::Rect::from_two_pos(self.pos, self.pos + self.image.size);
        let mut widget = Button::new(self.image.clone(), self.clicked.val(), rect);
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
        let response = ui.allocate_rect(self.rect, self.sense);

        if ui.is_rect_visible(self.rect) {
            let w = self.rect.max.x - 2.0 - self.rect.min.x + 2.0;

            if self.bipolar {
                let color = egui::Color32::from_rgb(0x33, 0x3f, 0x32);
                if self.value >= 0.5 {
                    ui.painter().rect_filled(
                        egui::Rect::from_two_pos(
                            egui::pos2(0.0, 0.0),
                            egui::pos2(w / 2.0 * (self.value as f32 - 0.5) * 2.0, 14.0),
                        )
                        .translate(egui::vec2(self.rect.min.x + w / 2.0, self.rect.min.y)),
                        egui::Rounding::none(),
                        color,
                    );
                } else {
                    let ratio = self.value as f32 * 2.0;
                    ui.painter().rect_filled(
                        egui::Rect::from_two_pos(
                            egui::pos2(0.0, 0.0),
                            egui::pos2(w / 2.0 * (1.0 - ratio), 14.0),
                        )
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
                    egui::Rect::from_two_pos(egui::pos2(0.0, 0.0), egui::pos2(2.0, 10.0))
                        .translate(egui::vec2(
                            self.rect.min.x + (w / 2.0 - 1.0),
                            self.rect.min.y + 2.0,
                        )),
                    egui::Rounding::none(),
                    egui::Color32::from_rgb(0x33, 0x3f, 0x32),
                );
            } else {
                ui.painter().rect_filled(
                    egui::Rect::from_two_pos(
                        self.rect.min,
                        egui::pos2(self.rect.min.x + w * self.value as f32, self.rect.max.y),
                    ),
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
    pos: egui::Pos2,
    parameter: SoyBoyParameter,
    param_def: ParameterDef,
    event_handler: Arc<dyn EventHandler>,
}

impl SliderBehavior {
    pub fn new(
        border_img: Image,
        value: f64,
        bipolar: bool,
        pos: egui::Pos2,
        parameter: SoyBoyParameter,
        param_def: ParameterDef,
        event_handler: Arc<dyn EventHandler>,
    ) -> Self {
        Self {
            border_img: border_img,
            value: value,
            bipolar: bipolar,
            pos,
            parameter,
            param_def,
            event_handler,
        }
    }
}

impl Behavior for SliderBehavior {
    fn update(&mut self) -> bool {
        false
    }

    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let rect = egui::Rect::from_two_pos(self.pos, self.pos + self.border_img.size);
        let widget = Slider::new(
            self.border_img.clone(),
            self.param_def
                .normalize(self.param_def.denormalize(self.value)),
            self.bipolar,
            rect,
        );
        let response = ui.add(widget);

        let delta_factor = if ui.input().modifiers.shift {
            // It may be wrong this way...
            3000.0
        } else {
            300.0
        };

        if response.dragged_by(egui::PointerButton::Primary) {
            let delta_x = response.drag_delta().x;
            let delta_v = delta_x as f64 / delta_factor;
            self.value = num::clamp(self.value + delta_v, 0.0, 1.0);
            self.event_handler
                .change_parameter(self.parameter, self.value);
        }

        response
    }
}

//    #[derive(Clone)]
pub struct ParameterSlider {
    slider: SliderBehavior,
    param: SoyBoyParameter,
    param_def: ParameterDef,
    unit: ParameterUnit,
    param_atlas: Image,
    value_atlas: Image,
    pos: egui::Pos2,
}

impl ParameterSlider {
    pub fn new(
        param: SoyBoyParameter,
        param_def: ParameterDef,
        value: f64,
        bipolar: bool,
        unit: ParameterUnit,
        border_img: Image,
        param_atlas: Image,
        value_atlas: Image,
        x: f32,
        y: f32,
        event_handler: Arc<dyn EventHandler>,
    ) -> Self {
        Self {
            param,
            param_def: param_def.clone(),
            unit,
            slider: SliderBehavior::new(
                border_img,
                value,
                bipolar,
                egui::pos2(x, y + 16.0),
                param,
                param_def,
                event_handler,
            ),
            param_atlas,
            value_atlas,
            pos: egui::pos2(x, y),
        }
    }
}

impl Behavior for ParameterSlider {
    fn update(&mut self) -> bool {
        self.slider.update()
    }

    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let rect = egui::Rect::from_two_pos(self.pos, self.pos + egui::vec2(266.0, 30.0));

        ui.set_clip_rect(rect);
        ui.add(ParameterName::new(
            self.param.clone(),
            self.param_atlas.clone(),
            self.pos,
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
        let value_rect = value.size;
        value.set_pos(self.pos + egui::vec2(size.x - value_rect.x, 0.0));
        ui.add(value);

        ui.set_clip_rect(screen_rect());

        self.slider.show(ui)
    }
}

pub struct SelectButton {
    param: SoyBoyParameter,
    value: usize,
    image: Image,
    pos: egui::Pos2,
}

impl SelectButton {
    pub fn new(param: SoyBoyParameter, value: usize, image: Image, x: f32, y: f32) -> Self {
        Self {
            param,
            value,
            image,
            pos: egui::pos2(x, y),
        }
    }
}

impl Widget for SelectButton {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let rect = egui::Rect::from_two_pos(self.pos, self.pos + self.image.size);

        let sense = egui::Sense {
            click: false,
            drag: false,
            focusable: false,
        };
        let response = ui.allocate_rect(rect, sense);

        if ui.is_rect_visible(rect) {
            let img = egui::widgets::Image::new(self.image.texture_id, self.image.size);
            img.paint_at(ui, rect);

            let regions = self.param.get_select_button_regions().unwrap();
            let region = regions.iter().nth(self.value).unwrap();
            let topleft = self.pos + region.pos.to_vec2();
            let selected_rect = egui::Rect::from_two_pos(topleft, topleft + region.size);
            ui.painter().rect_filled(
                selected_rect,
                egui::Rounding::none(),
                egui::Color32::from_rgba_unmultiplied(0x33, 0x3f, 0x32, 80),
            );
        }

        response
    }
}

pub struct ParameterSelector {
    param: SoyBoyParameter,
    param_def: ParameterDef,
    value: usize,
    button_image: Image,
    param_atlas: Image,
    pos: egui::Pos2,
    event_handler: Arc<dyn EventHandler>,
}

impl ParameterSelector {
    pub fn new(
        param: SoyBoyParameter,
        param_def: ParameterDef,
        value: f64,
        button_image: Image,
        param_atlas: Image,
        x: f32,
        y: f32,
        event_handler: Arc<dyn EventHandler>,
    ) -> Self {
        Self {
            param,
            param_def,
            value: value as usize,
            button_image,
            param_atlas,
            pos: egui::pos2(x, y),
            event_handler,
        }
    }
}

impl Behavior for ParameterSelector {
    fn update(&mut self) -> bool {
        false
    }

    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.add(ParameterName::new(
            self.param.clone(),
            self.param_atlas.clone(),
            self.pos,
        ));

        let topleft = self.pos + egui::vec2(0.0, 16.0);
        let button_rect = egui::Rect {
            min: topleft,
            max: topleft + self.button_image.size,
        };
        ui.set_clip_rect(button_rect);
        let _ = SelectButton::new(
            self.param,
            self.value,
            self.button_image.clone(),
            topleft.x,
            topleft.y,
        )
        .ui(ui);

        let responses: Vec<egui::Response> = self
            .param
            .get_select_button_regions()
            .unwrap()
            .iter()
            .map(|reg| {
                let topleft = topleft + reg.pos.to_vec2();
                ui.allocate_rect(
                    egui::Rect {
                        min: topleft,
                        max: topleft + reg.size,
                    },
                    egui::Sense::click(),
                )
            })
            .collect();

        if let Some(pos) = responses.iter().position(|res| res.clicked()) {
            self.value = pos;
            self.event_handler
                .change_parameter(self.param, self.param_def.normalize(self.value as f64));
        }

        responses[self.value].clone()
    }
}
