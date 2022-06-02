use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use egui_glow::egui_winit::egui;

use crate::common::PluginConfigV02;
use crate::gui::images::{Image, Images};
use crate::soyboy::parameters::{ParameterDef, Parametric, SoyBoyParameter};
use crate::vst3::ControllerConnection;

use super::{images, types::*, widget::*};

pub struct UI {
    _images: Images,
    pub edamame: AnimatedEdamame,
    pub version: VersionFrame,
    pub label_logo: ImageLabel,
    pub label_global: ImageLabel,
    pub label_square: ImageLabel,
    pub label_noise: ImageLabel,
    pub label_wavetable: ImageLabel,
    pub label_envelope: ImageLabel,
    pub label_sweep: ImageLabel,
    pub label_stutter: ImageLabel,
    pub oscilloscope: Oscilloscope,
    pub button_reset_random: ButtonBehavior,
    pub button_reset_sine: ButtonBehavior,
    pub param_volume: ParameterSlider,
    pub param_detune: ParameterSlider,
    pub param_interval: ParameterSlider,
    pub param_attack: ParameterSlider,
    pub param_decay: ParameterSlider,
    pub param_sustain: ParameterSlider,
    pub param_release: ParameterSlider,
    pub param_amount: ParameterSlider,
    pub param_period: ParameterSlider,
    pub param_time: ParameterSlider,
    pub param_depth: ParameterSlider,
    pub param_osc_type: ParameterSelector,
    pub param_osc_sq_duty: ParameterSelector,
    pub param_sweep_type: ParameterSelector,
    pub param_voices: ParameterVoices,
    pub param_wavetable: WaveTableEditor,
}

impl UI {
    pub fn new(
        egui_ctx: &egui::Context,
        param_defs: HashMap<SoyBoyParameter, ParameterDef>,
        param_values: Arc<Mutex<HashMap<u32, f64>>>,
        event_handler: Arc<dyn EventHandler>,
        controller_connection: Arc<Mutex<ControllerConnection>>,
        waveform_view_enabled: Rc<RefCell<bool>>,
    ) -> Self {
        let images = images::Images::new();

        let slider_images = SliderImages {
            border_img: Image::new(egui_ctx, &images.slider_border),
            param_atlas: Image::new(egui_ctx, &images.param_atlas),
            value_atlas: Image::new(egui_ctx, &images.value_atlas),
        };

        let param_values = param_values.lock().unwrap();
        Self {
            edamame: AnimatedEdamame::new(Image::new(egui_ctx, &images.edamame), 18.0, 14.0),
            version: VersionFrame::new(
                Image::new(egui_ctx, &images.version_frame),
                Image::new(egui_ctx, &images.value_atlas),
                148.0,
                58.0,
            ),
            label_logo: ImageLabel::new(Image::new(egui_ctx, &images.label_logo), 6.0, 6.0),
            label_global: ImageLabel::new(Image::new(egui_ctx, &images.label_global), 24.0, 86.0),
            label_square: ImageLabel::new(Image::new(egui_ctx, &images.label_square), 24.0, 216.0),
            label_noise: ImageLabel::new(Image::new(egui_ctx, &images.label_noise), 24.0, 280.0),
            label_wavetable: ImageLabel::new(
                Image::new(egui_ctx, &images.label_wavetable),
                24.0,
                408.0,
            ),
            label_envelope: ImageLabel::new(
                Image::new(egui_ctx, &images.label_envelope),
                352.0,
                12.0,
            ),
            label_sweep: ImageLabel::new(Image::new(egui_ctx, &images.label_sweep), 352.0, 184.0),
            label_stutter: ImageLabel::new(
                Image::new(egui_ctx, &images.label_stutter),
                352.0,
                316.0,
            ),
            oscilloscope: Oscilloscope::new(
                waveform_view_enabled.clone(),
                Image::new(egui_ctx, &images.oscilloscope_border),
                352.0,
                460.0,
                controller_connection.clone(),
            ),
            button_reset_random: ButtonBehavior::new(
                Image::new(egui_ctx, &images.button_reset_random),
                206.0,
                526.0,
            ),
            button_reset_sine: ButtonBehavior::new(
                Image::new(egui_ctx, &images.button_reset_sine),
                274.0,
                526.0,
            ),
            param_volume: ParameterSlider::new(
                SliderValue {
                    param: SoyBoyParameter::MasterVolume,
                    param_def: param_defs
                        .get(&SoyBoyParameter::MasterVolume)
                        .unwrap()
                        .clone(),
                    value: *param_values
                        .get(&(SoyBoyParameter::MasterVolume as u32))
                        .unwrap(),
                    bipolar: false,
                    unit: ParameterUnit::Decibel,
                },
                slider_images,
                60.0,
                86.0 + 2.0,
                event_handler.clone(),
            ),
            param_detune: ParameterSlider::new(
                SliderValue {
                    param: SoyBoyParameter::Detune,
                    param_def: param_defs.get(&SoyBoyParameter::Detune).unwrap().clone(),
                    value: *param_values.get(&(SoyBoyParameter::Detune as u32)).unwrap(),
                    bipolar: true,
                    unit: ParameterUnit::Cent,
                },
                slider_images,
                60.0,
                122.0 + 2.0,
                event_handler.clone(),
            ),
            param_interval: ParameterSlider::new(
                SliderValue {
                    param: SoyBoyParameter::OscNsInterval,
                    param_def: param_defs
                        .get(&SoyBoyParameter::OscNsInterval)
                        .unwrap()
                        .clone(),
                    value: *param_values
                        .get(&(SoyBoyParameter::OscNsInterval as u32))
                        .unwrap(),
                    bipolar: false,
                    unit: ParameterUnit::MilliSec,
                },
                slider_images,
                60.0,
                292.0 + 2.0,
                event_handler.clone(),
            ),
            param_attack: ParameterSlider::new(
                SliderValue {
                    param: SoyBoyParameter::EgAttack,
                    param_def: param_defs.get(&SoyBoyParameter::EgAttack).unwrap().clone(),
                    value: *param_values
                        .get(&(SoyBoyParameter::EgAttack as u32))
                        .unwrap(),
                    bipolar: false,
                    unit: ParameterUnit::Sec,
                },
                slider_images,
                388.0,
                24.0 + 2.0,
                event_handler.clone(),
            ),
            param_decay: ParameterSlider::new(
                SliderValue {
                    param: SoyBoyParameter::EgDecay,
                    param_def: param_defs.get(&SoyBoyParameter::EgDecay).unwrap().clone(),
                    value: *param_values
                        .get(&(SoyBoyParameter::EgDecay as u32))
                        .unwrap(),
                    bipolar: false,
                    unit: ParameterUnit::Sec,
                },
                slider_images,
                388.0,
                58.0 + 2.0,
                event_handler.clone(),
            ),
            param_sustain: ParameterSlider::new(
                SliderValue {
                    param: SoyBoyParameter::EgSustain,
                    param_def: param_defs.get(&SoyBoyParameter::EgSustain).unwrap().clone(),
                    value: *param_values
                        .get(&(SoyBoyParameter::EgSustain as u32))
                        .unwrap(),
                    bipolar: false,
                    unit: ParameterUnit::None,
                },
                slider_images,
                388.0,
                92.0 + 2.0,
                event_handler.clone(),
            ),
            param_release: ParameterSlider::new(
                SliderValue {
                    param: SoyBoyParameter::EgRelease,
                    param_def: param_defs.get(&SoyBoyParameter::EgRelease).unwrap().clone(),
                    value: *param_values
                        .get(&(SoyBoyParameter::EgRelease as u32))
                        .unwrap(),
                    bipolar: false,
                    unit: ParameterUnit::Sec,
                },
                slider_images,
                388.0,
                126.0 + 2.0,
                event_handler.clone(),
            ),
            param_amount: ParameterSlider::new(
                SliderValue {
                    param: SoyBoyParameter::SweepAmount,
                    param_def: param_defs
                        .get(&SoyBoyParameter::SweepAmount)
                        .unwrap()
                        .clone(),
                    value: *param_values
                        .get(&(SoyBoyParameter::SweepAmount as u32))
                        .unwrap(),
                    bipolar: false,
                    unit: ParameterUnit::None,
                },
                slider_images,
                388.0,
                232.0,
                event_handler.clone(),
            ),
            param_period: ParameterSlider::new(
                SliderValue {
                    param: SoyBoyParameter::SweepPeriod,
                    param_def: param_defs
                        .get(&SoyBoyParameter::SweepPeriod)
                        .unwrap()
                        .clone(),
                    value: *param_values
                        .get(&(SoyBoyParameter::SweepPeriod as u32))
                        .unwrap(),
                    bipolar: false,
                    unit: ParameterUnit::None,
                },
                slider_images,
                388.0,
                268.0,
                event_handler.clone(),
            ),
            param_time: ParameterSlider::new(
                SliderValue {
                    param: SoyBoyParameter::StutterTime,
                    param_def: param_defs
                        .get(&SoyBoyParameter::StutterTime)
                        .unwrap()
                        .clone(),
                    value: *param_values
                        .get(&(SoyBoyParameter::StutterTime as u32))
                        .unwrap(),
                    bipolar: false,
                    unit: ParameterUnit::Sec,
                },
                slider_images,
                388.0,
                342.0,
                event_handler.clone(),
            ),
            param_depth: ParameterSlider::new(
                SliderValue {
                    param: SoyBoyParameter::StutterDepth,
                    param_def: param_defs
                        .get(&SoyBoyParameter::StutterDepth)
                        .unwrap()
                        .clone(),
                    value: *param_values
                        .get(&(SoyBoyParameter::StutterDepth as u32))
                        .unwrap(),
                    bipolar: false,
                    unit: ParameterUnit::Percent,
                },
                slider_images,
                388.0,
                378.0,
                event_handler.clone(),
            ),
            param_osc_type: ParameterSelector::new(
                SelectorValue {
                    param: SoyBoyParameter::OscillatorType,
                    param_def: param_defs
                        .get(&SoyBoyParameter::OscillatorType)
                        .unwrap()
                        .clone(),
                    value: *param_values
                        .get(&(SoyBoyParameter::OscillatorType as u32))
                        .unwrap(),
                },
                Image::new(egui_ctx, &images.select_osc_type),
                Image::new(egui_ctx, &images.param_atlas),
                60.0,
                159.0,
                event_handler.clone(),
            ),
            param_osc_sq_duty: ParameterSelector::new(
                SelectorValue {
                    param: SoyBoyParameter::OscSqDuty,
                    param_def: param_defs.get(&SoyBoyParameter::OscSqDuty).unwrap().clone(),
                    value: *param_values
                        .get(&(SoyBoyParameter::OscSqDuty as u32))
                        .unwrap(),
                },
                Image::new(egui_ctx, &images.select_osc_sq_duty),
                Image::new(egui_ctx, &images.param_atlas),
                60.0,
                220.0,
                event_handler.clone(),
            ),
            param_sweep_type: ParameterSelector::new(
                SelectorValue {
                    param: SoyBoyParameter::SweepType,
                    param_def: param_defs.get(&SoyBoyParameter::SweepType).unwrap().clone(),
                    value: *param_values
                        .get(&(SoyBoyParameter::SweepType as u32))
                        .unwrap(),
                },
                Image::new(egui_ctx, &images.select_sweep_type),
                Image::new(egui_ctx, &images.param_atlas),
                388.0,
                186.0,
                event_handler.clone(),
            ),
            param_voices: ParameterVoices::new(
                *param_values
                    .get(&(SoyBoyParameter::NumVoices as u32))
                    .unwrap(),
                param_defs.get(&SoyBoyParameter::NumVoices).unwrap().clone(),
                Image::new(egui_ctx, &images.value_atlas),
                Image::new(egui_ctx, &images.button_minus),
                Image::new(egui_ctx, &images.button_plus),
                252.0,
                158.0,
                event_handler.clone(),
            ),
            param_wavetable: WaveTableEditor::new(
                Image::new(egui_ctx, &images.wavetable_border),
                60.0,
                340.0,
                controller_connection,
            ),
            _images: images,
        }
    }

    pub fn set_value(&mut self, param: &SoyBoyParameter, value: f64) {
        match param {
            SoyBoyParameter::MasterVolume => self.param_volume.set(value),
            SoyBoyParameter::PitchBend => (),
            SoyBoyParameter::Detune => self.param_detune.set(value),
            SoyBoyParameter::OscillatorType => self.param_osc_type.set(value),
            SoyBoyParameter::NumVoices => self.param_voices.set(value),
            SoyBoyParameter::SweepType => self.param_sweep_type.set(value),
            SoyBoyParameter::SweepAmount => self.param_amount.set(value),
            SoyBoyParameter::SweepPeriod => self.param_period.set(value),
            SoyBoyParameter::StutterTime => self.param_time.set(value),
            SoyBoyParameter::StutterDepth => self.param_depth.set(value),
            SoyBoyParameter::StutterWhen => (), //self.param_when.set(value),
            SoyBoyParameter::EgAttack => self.param_attack.set(value),
            SoyBoyParameter::EgDecay => self.param_decay.set(value),
            SoyBoyParameter::EgSustain => self.param_sustain.set(value),
            SoyBoyParameter::EgRelease => self.param_release.set(value),
            SoyBoyParameter::OscSqDuty => self.param_osc_sq_duty.set(value),
            SoyBoyParameter::OscNsInterval => self.param_interval.set(value),
            SoyBoyParameter::DacFreq => (),
            SoyBoyParameter::DacQ => (),
        }
    }

    pub fn configure(&mut self, config: PluginConfigV02) {
        for ref param in SoyBoyParameter::iter() {
            self.set_value(param, config.get_param(param));
        }

        self.param_wavetable.set_wavetable(&config.wavetable);
    }
}
