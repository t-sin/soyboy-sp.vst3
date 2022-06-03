use std::os::raw::c_void;

use egui_glow::egui_winit::egui;

use crate::soyboy::parameters::SoyBoyParameter;

pub struct ParentWindow(pub *mut c_void);
unsafe impl Send for ParentWindow {}
unsafe impl Sync for ParentWindow {}

pub trait EventHandler {
    fn change_parameter(&self, p: SoyBoyParameter, value_normalized: f64);
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

// available characters in resources/paramval.png
pub enum Character {
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

impl Character {
    pub fn from_char(ch: char) -> Option<Character> {
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

    pub fn get_region(&self) -> Region {
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
    Voices,
}

impl ParameterUnit {
    pub fn get_region(&self) -> Option<Region> {
        match self {
            ParameterUnit::None => None,
            ParameterUnit::Decibel => Some(Region::new(0.0, 16.0, 22.0, 14.0)),
            ParameterUnit::Cent => Some(Region::new(30.0, 16.0, 58.0, 14.0)),
            ParameterUnit::MilliSec => Some(Region::new(96.0, 16.0, 22.0, 14.0)),
            ParameterUnit::Sec => Some(Region::new(126.0, 16.0, 10.0, 14.0)),
            ParameterUnit::Percent => Some(Region::new(144.0, 16.0, 10.0, 14.0)),
            ParameterUnit::Voices => Some(Region::new(0.0, 32.0, 62.0, 14.0)),
        }
    }
}

// available parameters in resources/paramval.png
impl SoyBoyParameter {
    pub fn get_region(&self) -> Option<Region> {
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
            SoyBoyParameter::StutterWhen => Some(Region::new(0.0, 224.0, 53.0, 14.0)),
            _ => None,
        }
    }
}

impl SoyBoyParameter {
    pub fn get_select_button_regions(&self) -> Option<Vec<Region>> {
        match self {
            SoyBoyParameter::OscillatorType => Some(vec![
                Region::new(2.0, 2.0, 58.0, 22.0),
                Region::new(62.0, 2.0, 58.0, 22.0),
                Region::new(122.0, 2.0, 54.0, 22.0),
            ]),
            SoyBoyParameter::OscSqDuty => Some(vec![
                Region::new(2.0, 2.0, 46.0, 22.0),
                Region::new(50.0, 2.0, 40.0, 22.0),
                Region::new(92.0, 2.0, 42.0, 22.0),
                Region::new(136.0, 2.0, 40.0, 22.0),
            ]),
            SoyBoyParameter::SweepType => Some(vec![
                Region::new(2.0, 2.0, 62.0, 20.0),
                Region::new(66.0, 2.0, 48.0, 20.0),
                Region::new(116.0, 2.0, 64.0, 20.0),
                Region::new(182.0, 2.0, 58.0, 20.0),
            ]),
            SoyBoyParameter::StutterWhen => Some(vec![
                Region::new(2.0, 2.0, 92.0, 20.0),
                Region::new(96.0, 2.0, 84.0, 20.0),
            ]),
            _ => None,
        }
    }
}
