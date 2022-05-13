use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time;

use egui_glow::egui_winit::{egui, egui::Widget};

use crate::common::{constants, i4, Vst3Message};
use crate::gui::{images::Image, types::*};
use crate::soyboy::parameters::{Normalizable, ParameterDef, SoyBoyParameter};
use crate::ControllerConnection;

fn screen_rect() -> egui::Rect {
    egui::Rect {
        min: egui::pos2(0.0, 0.0),
        max: egui::pos2(
            constants::SCREEN_WIDTH as f32,
            constants::SCREEN_HEIGHT as f32,
        ),
    }
}

#[derive(Clone)]
pub struct ParameterValue {
    atlas: Image,
    regions: Vec<Region>,
    rect: egui::Rect,
}

impl ParameterValue {
    pub fn new(value_str: String, unit: ParameterUnit, atlas: Image, x: f32, y: f32) -> Self {
        let (regions, w, h) = ParameterValue::layout(value_str, unit);

        let pos = egui::pos2(x, y);
        let size = egui::vec2(w, h);
        let rect = egui::Rect::from_two_pos(pos, pos + size);
        Self {
            atlas,
            regions,
            rect,
        }
    }

    pub fn set_pos(&mut self, pos: egui::Pos2) {
        let size = self.rect.size();
        self.rect = egui::Rect::from_min_size(pos, size);
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
        let response = ui.allocate_rect(self.rect, egui::Sense::focusable_noninteractive());

        if ui.is_rect_visible(self.rect) {
            let mut offset = egui::pos2(0.0, 0.0);
            let img = egui::widgets::Image::new(self.atlas.texture_id, self.atlas.size);

            for region in self.regions.iter() {
                let clip_rect = egui::Rect::from_min_size(self.rect.min, region.size);
                ui.set_clip_rect(clip_rect.translate(offset.to_vec2()));

                let draw_rect = egui::Rect::from_min_size(self.rect.min, self.atlas.size);

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
    region: Region,
    rect: egui::Rect,
    draw_rect: egui::Rect,
    image: egui::widgets::Image,
}

impl ParameterName {
    pub fn new(param: SoyBoyParameter, atlas: Image, pos: egui::Pos2) -> Self {
        let region = param.get_region().unwrap();
        let rect = egui::Rect::from_min_size(pos, region.size);
        let draw_rect = egui::Rect::from_min_size(rect.min, atlas.size);
        let image = egui::widgets::Image::new(atlas.texture_id, atlas.size);
        Self {
            region,
            rect,
            draw_rect,
            image,
        }
    }
}

impl Widget for ParameterName {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.set_clip_rect(self.rect);

        let response = ui.allocate_rect(self.rect, egui::Sense::focusable_noninteractive());

        if ui.is_rect_visible(self.rect) {
            self.image
                .paint_at(ui, self.draw_rect.translate(-self.region.pos.to_vec2()));
        }

        response
    }
}

#[derive(Clone)]
pub struct VersionFrame {
    rect: egui::Rect,
    border_image: egui::widgets::Image,
    param_value: ParameterValue,
}

impl VersionFrame {
    pub fn new(frame: Image, atlas: Image, x: f32, y: f32) -> Self {
        let pos = egui::pos2(x, y);
        let rect = egui::Rect::from_min_size(pos, frame.size);
        let border_image = egui::widgets::Image::new(frame.texture_id, frame.size);
        let value_pos = rect.min + egui::vec2(6.0, 8.0);
        let param_value = ParameterValue::new(
            env!("CARGO_PKG_VERSION").to_string(),
            ParameterUnit::None,
            atlas,
            value_pos.x,
            value_pos.y,
        );

        Self {
            rect,
            border_image,
            param_value,
        }
    }
}

impl Widget for VersionFrame {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.set_clip_rect(self.rect);

        let response = ui.allocate_rect(self.rect, egui::Sense::focusable_noninteractive());

        if ui.is_rect_visible(self.rect) {
            self.border_image.paint_at(ui, self.rect);
            let _ = ui.add(self.param_value);
        }

        response
    }
}

#[derive(Clone)]
pub struct ImageLabel {
    image: egui::widgets::Image,
    sense: egui::Sense,
    rect: egui::Rect,
}

impl ImageLabel {
    pub fn new(image: Image, x: f32, y: f32) -> Self {
        let pos = egui::pos2(x, y);
        let rect = egui::Rect::from_min_size(pos, image.size);
        let image = egui::widgets::Image::new(image.texture_id, rect.size());

        Self {
            image,
            sense: egui::Sense::focusable_noninteractive(),
            rect,
        }
    }
}

impl Widget for ImageLabel {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.set_clip_rect(self.rect);

        let response = ui.allocate_rect(self.rect, self.sense);

        if ui.is_rect_visible(self.rect) {
            self.image.paint_at(ui, self.rect);
        }

        response
    }
}

#[derive(Copy, Clone)]
pub struct Edamame {
    image: egui::widgets::Image,
    jumping: bool,
    sense: egui::Sense,
    rect: egui::Rect,
    clip_rect: egui::Rect,
    jumping_rect: egui::Rect,
}

impl Edamame {
    pub fn new(image: Image, jumping: bool, pos: egui::Pos2) -> Self {
        let rect = egui::Rect::from_min_size(pos, egui::vec2(image.size.x, image.size.y));
        let image = egui::widgets::Image::new(image.texture_id, rect.size());

        let half_x = image.size().x / 2.0;
        let clip_rect = egui::Rect::from_min_size(pos, egui::vec2(half_x, image.size().y));
        let jumping_rect = rect.translate(egui::vec2(-half_x, 0.0));

        Self {
            image,
            jumping,
            sense: egui::Sense::click(),
            rect,
            clip_rect,
            jumping_rect,
        }
    }
}

impl Widget for Edamame {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        if ui.is_rect_visible(self.clip_rect) {
            ui.set_clip_rect(self.clip_rect);

            if self.jumping {
                self.image.paint_at(ui, self.jumping_rect);
            } else {
                self.image.paint_at(ui, self.rect);
            }

            ui.set_clip_rect(screen_rect());
        }

        ui.allocate_rect(self.clip_rect, self.sense)
    }
}

#[derive(Clone)]
pub struct AnimatedEdamame {
    edamame: Edamame,
    jumped_at: time::Instant,
    jumping: Toggle,
}

impl AnimatedEdamame {
    pub fn new(image: Image, x: f32, y: f32) -> Self {
        let pos = egui::pos2(x, y);
        let edamame = Edamame::new(image, false, pos);

        Self {
            edamame,
            jumping: Toggle::new(false, false),
            jumped_at: time::Instant::now(),
        }
    }

    pub fn jump(&mut self) {
        self.jumped_at = time::Instant::now();
    }
}

impl Behavior for AnimatedEdamame {
    fn update(&mut self) -> bool {
        if self.jumped_at.elapsed() <= time::Duration::from_millis(80) {
            self.jumping.set(true);
        } else {
            self.jumping.set(false);
        }

        self.jumping.toggled()
    }

    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        self.edamame.jumping = self.jumping.val();
        let response = ui.add(self.edamame);

        if response.clicked() {
            self.jump();
        }

        response
    }
}

#[derive(Clone)]
pub struct Button {
    image: egui::widgets::Image,
    sense: egui::Sense,
    clicked: bool,
    rect: egui::Rect,
    clicked_rect: egui::Rect,
}

impl Button {
    pub fn new(image: Image, clicked: bool, rect: egui::Rect) -> Self {
        let image = egui::widgets::Image::new(image.texture_id, rect.size());
        let clicked_rect = rect.translate(egui::vec2(2.0, 2.0));

        Self {
            image,
            sense: egui::Sense::click().union(egui::Sense::hover()),
            clicked,
            rect,
            clicked_rect,
        }
    }
}

impl Widget for &mut Button {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let rect = if self.clicked {
            self.clicked_rect
        } else {
            self.rect
        };

        let response = ui.allocate_rect(rect, self.sense);

        if ui.is_rect_visible(rect) {
            self.image.paint_at(ui, rect);

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
    button: Button,
    clicked_at: time::Instant,
    clicked: Toggle,
}

impl ButtonBehavior {
    pub fn new(image: Image, x: f32, y: f32) -> Self {
        let pos = egui::pos2(x, y);
        let rect = egui::Rect::from_min_size(pos, image.size);
        let button = Button::new(image, false, rect);

        Self {
            button,
            clicked_at: time::Instant::now(),
            clicked: Toggle::new(false, false),
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
        self.button.clicked = self.clicked.val();
        let response = self.button.ui(ui);

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
            border_img,
            sense: egui::Sense::drag(),
            rect,
            bipolar,
            value,
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
    rect: egui::Rect,
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
        let rect = egui::Rect::from_min_size(pos, border_img.size);

        Self {
            border_img,
            value,
            bipolar,
            rect,
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
        let widget = Slider::new(
            self.border_img,
            self.param_def
                .normalize(self.param_def.denormalize(self.value)),
            self.bipolar,
            self.rect,
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

#[derive(Copy, Clone)]
pub struct SliderImages {
    pub border_img: Image,
    pub param_atlas: Image,
    pub value_atlas: Image,
}

pub struct SliderValue {
    pub param: SoyBoyParameter,
    pub param_def: ParameterDef,
    pub value: f64,
    pub bipolar: bool,
    pub unit: ParameterUnit,
}

impl ParameterSlider {
    pub fn new(
        value: SliderValue,
        images: SliderImages,
        x: f32,
        y: f32,
        event_handler: Arc<dyn EventHandler>,
    ) -> Self {
        Self {
            param: value.param,
            param_def: value.param_def.clone(),
            unit: value.unit,
            slider: SliderBehavior::new(
                images.border_img,
                value.value,
                value.bipolar,
                egui::pos2(x, y + 16.0),
                value.param,
                value.param_def,
                event_handler,
            ),
            param_atlas: images.param_atlas,
            value_atlas: images.value_atlas,
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
        ui.add(ParameterName::new(self.param, self.param_atlas, self.pos));
        ui.set_clip_rect(rect);

        let mut value = ParameterValue::new(
            self.param_def.format(self.slider.value),
            self.unit.clone(),
            self.value_atlas,
            0.0,
            0.0,
        );
        let size = egui::vec2(266.0, 30.0);
        let value_rect = value.rect.size();
        value.set_pos(self.pos + egui::vec2(size.x - value_rect.x, 0.0));
        ui.add(value);

        ui.set_clip_rect(screen_rect());

        self.slider.show(ui)
    }
}

pub struct SelectButton {
    param: SoyBoyParameter,
    value: usize,
    image: egui::widgets::Image,
    rect: egui::Rect,
}

impl SelectButton {
    pub fn new(param: SoyBoyParameter, value: usize, image: Image, x: f32, y: f32) -> Self {
        let pos = egui::pos2(x, y);
        let rect = egui::Rect::from_min_size(pos, image.size);
        let image = egui::widgets::Image::new(image.texture_id, image.size);

        Self {
            param,
            value,
            image,
            rect,
        }
    }
}

impl Widget for SelectButton {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let sense = egui::Sense {
            click: false,
            drag: false,
            focusable: false,
        };
        let response = ui.allocate_rect(self.rect, sense);

        if ui.is_rect_visible(self.rect) {
            self.image.paint_at(ui, self.rect);

            let regions = self.param.get_select_button_regions().unwrap();
            let region = regions.get(self.value).unwrap();
            let min = self.rect.min + region.pos.to_vec2();
            let selected_rect = egui::Rect::from_min_size(min, region.size);
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

pub struct SelectorValue {
    pub param: SoyBoyParameter,
    pub param_def: ParameterDef,
    pub value: f64,
}

impl ParameterSelector {
    pub fn new(
        value: SelectorValue,
        button_image: Image,
        param_atlas: Image,
        x: f32,
        y: f32,
        event_handler: Arc<dyn EventHandler>,
    ) -> Self {
        let v = value.param_def.denormalize(value.value) as usize;
        Self {
            param: value.param,
            param_def: value.param_def,
            value: v,
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
        ui.add(ParameterName::new(self.param, self.param_atlas, self.pos));

        let topleft = self.pos + egui::vec2(0.0, 16.0);
        let button_rect = egui::Rect {
            min: topleft,
            max: topleft + self.button_image.size,
        };
        ui.set_clip_rect(button_rect);
        let _ = SelectButton::new(
            self.param,
            self.value,
            self.button_image,
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

pub struct WaveTableEditor {
    values: [f64; constants::WAVETABLE_SIZE],
    border_image: egui::widgets::Image,
    rect: egui::Rect,
    controller_connection: Arc<Mutex<ControllerConnection>>,
}

impl WaveTableEditor {
    pub fn new(
        border_img: Image,
        x: f32,
        y: f32,
        controller_connection: Arc<Mutex<ControllerConnection>>,
    ) -> Self {
        let pos = egui::pos2(x, y);
        let rect = egui::Rect::from_min_size(pos, border_img.size);
        let border_image = egui::widgets::Image::new(border_img.texture_id, border_img.size);

        Self {
            values: [1.0; constants::WAVETABLE_SIZE],
            border_image,
            rect,
            controller_connection,
        }
    }

    pub fn set_wavetable(&mut self, samples: &[i4; constants::WAVETABLE_SIZE]) {
        for (i, v) in self.values.iter_mut().enumerate() {
            let s: i8 = samples[i].into();
            *v = (s + i4::SIGNED_MIN.abs()) as f64 / i4::LEVELS as f64;
        }
    }

    fn show_sample_slider(ui: &mut egui::Ui, rect: egui::Rect, value: f64) {
        let color = egui::Color32::from_rgb(0x4f, 0x5e, 0x4d);

        if ui.is_rect_visible(rect) {
            let value = ((value * (i4::LEVELS as f64)) as usize) as f64 / (i4::LEVELS as f64);

            let slider_height = 166.0;
            let center_y = rect.min.y + rect.size().y / 2.0;
            let center_pos = egui::pos2(rect.min.x, center_y);

            if value > 0.5 {
                let size = egui::vec2(6.0, -(value - 0.5) as f32 * slider_height);
                ui.painter().rect_filled(
                    egui::Rect::from_two_pos(center_pos, center_pos + size),
                    egui::Rounding::none(),
                    color,
                );
            } else {
                let size = egui::vec2(6.0, (0.5 - value) as f32 * slider_height);
                ui.painter().rect_filled(
                    egui::Rect::from_two_pos(center_pos, center_pos + size),
                    egui::Rounding::none(),
                    color,
                );
            }
        }
    }
}

impl Behavior for WaveTableEditor {
    fn update(&mut self) -> bool {
        false
    }

    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.set_clip_rect(self.rect);
        let response = ui.allocate_rect(self.rect, egui::Sense::drag());

        let size = egui::vec2(6.0, 6.0 + 6.0 + 166.0);
        let size_y = size.y - 6.0 - 6.0;
        for (i, value) in self.values.iter_mut().enumerate() {
            let pos = self.rect.min + egui::vec2(6.0, 2.0) + egui::vec2(8.0 * i as f32, 0.0);
            let slider_rect = egui::Rect::from_two_pos(pos, pos + size);

            if response.dragged_by(egui::PointerButton::Primary) {
                if let Some(pointer_pos) = response.hover_pos() {
                    if slider_rect.contains(pointer_pos) {
                        let pointer_pos = pointer_pos - pos.to_vec2() - egui::vec2(0.0, 6.0);
                        let new_value = (size_y - pointer_pos.y - 3.0) / size_y;

                        let v = i4::from((new_value * i4::LEVELS as f32) as u8);
                        self.controller_connection
                            .lock()
                            .unwrap()
                            .send_message(Vst3Message::SetWaveTable(i, v));

                        *value = num::clamp(new_value as f64, 0.0, 1.0);
                    }
                }
            }

            Self::show_sample_slider(ui, slider_rect, *value);
        }

        self.border_image.paint_at(ui, self.rect);

        response
    }
}

pub struct Oscilloscope {
    signals: [f64; constants::OSCILLOSCOPE_SAIMPLE_SIZE],
    enabled: Rc<RefCell<bool>>,
    pos: egui::Pos2,
    border_img: Image,
    controller_connection: Arc<Mutex<ControllerConnection>>,
}

impl Oscilloscope {
    pub fn new(
        enabled: Rc<RefCell<bool>>,
        border_img: Image,
        x: f32,
        y: f32,
        controller_connection: Arc<Mutex<ControllerConnection>>,
    ) -> Self {
        Self {
            signals: [0.0; constants::OSCILLOSCOPE_SAIMPLE_SIZE],
            enabled,
            pos: egui::pos2(x, y),
            border_img,
            controller_connection,
        }
    }

    pub fn set_signals(&mut self, signals: &[f64]) {
        (&mut self.signals).copy_from_slice(&signals[..]);
    }
}

impl Behavior for Oscilloscope {
    fn update(&mut self) -> bool {
        false
    }

    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let rect = egui::Rect::from_two_pos(self.pos, self.pos + self.border_img.size);
        ui.set_clip_rect(rect);
        let response = ui.allocate_rect(rect, egui::Sense::click());

        if response.clicked() {
            let enabled = *self.enabled.borrow();
            *self.enabled.borrow_mut() = !enabled;

            let msg = if *self.enabled.borrow() {
                Vst3Message::EnableWaveform
            } else {
                Vst3Message::DisableWaveform
            };
            self.controller_connection.lock().unwrap().send_message(msg);
        }

        let img = egui::widgets::Image::new(self.border_img.texture_id, self.border_img.size);
        img.paint_at(ui, rect);

        if *self.enabled.borrow() {
            static SCALE_X: f32 = 1.0;
            let w = self.border_img.size.x;
            let h = 90.0;
            let hh = h / 2.0;
            let dx = w / constants::OSCILLOSCOPE_SAIMPLE_SIZE as f32 * SCALE_X;
            let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0x1c, 0x23, 0x1b));

            static SAMPLES_TO_DISPLAY: usize =
                constants::OSCILLOSCOPE_SAIMPLE_SIZE / SCALE_X as usize;
            let mut points = vec![egui::pos2(0.0, 0.0); SAMPLES_TO_DISPLAY];

            for i in 0..SAMPLES_TO_DISPLAY {
                let idx = i;

                let s = -self.signals[idx] as f32;
                let p = egui::pos2(self.pos.x + i as f32 * dx, s * h + self.pos.y + hh);
                points[i] = p;
            }

            ui.painter().add(egui::Shape::line(points, stroke));
        } else {
            ui.painter().rect_filled(
                rect,
                egui::Rounding::none(),
                egui::Color32::from_rgba_unmultiplied(0x33, 0x3f, 0x32, 130),
            );
        }

        response
    }
}
