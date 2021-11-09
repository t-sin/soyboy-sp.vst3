use std::cell::RefCell;
use std::convert::TryFrom;

use std::os::raw::c_void;

use vst3_com::{sys::GUID, ComPtr, IID};
use vst3_sys::{
    base::{kInvalidArgument, kResultFalse, kResultOk, kResultTrue, tresult, IPluginBase, TBool},
    vst::{
        AudioBusBuffers, BusDirections, BusFlags, BusInfo, BusTypes, EventTypes, IAudioProcessor,
        IComponent, IEventList, IParamValueQueue, IParameterChanges, MediaTypes, ProcessData,
        ProcessSetup, RoutingInfo, K_SAMPLE32, K_SAMPLE64,
    },
    VST3,
};

use crate::vst3::{controller::GameBoyController, plugin_data, util};

use crate::gbi::{AudioProcessor, GameBoyInstrument, Parameter, Parametric};

#[VST3(implements(IComponent, IAudioProcessor))]
pub struct GameBoyPlugin {
    gbi: RefCell<GameBoyInstrument>,
    audio_out: RefCell<BusInfo>,
    event_in: RefCell<BusInfo>,
}

impl GameBoyPlugin {
    pub const CID: GUID = GUID {
        data: plugin_data::VST3_CID,
    };

    unsafe fn init_event_in(&mut self) {
        let mut bus = self.event_in.borrow_mut();

        util::wstrcpy("Event In", bus.name.as_mut_ptr());
        bus.media_type = MediaTypes::kEvent as i32;
        bus.direction = BusDirections::kInput as i32;
        bus.channel_count = 1;
        bus.bus_type = BusTypes::kMain as i32;
        bus.flags = BusFlags::kDefaultActive as u32;
    }

    unsafe fn init_audio_out(&mut self) {
        let mut bus = self.audio_out.borrow_mut();

        util::wstrcpy("Audio Out", bus.name.as_mut_ptr());
        bus.media_type = MediaTypes::kAudio as i32;
        bus.direction = BusDirections::kOutput as i32;
        bus.channel_count = 2;
        bus.bus_type = BusTypes::kMain as i32;
        bus.flags = BusFlags::kDefaultActive as u32;
    }

    pub unsafe fn new(gbi: GameBoyInstrument) -> Box<Self> {
        let gbi = RefCell::new(gbi);
        let audio_out = RefCell::new(util::make_empty_bus_info());
        let event_in = RefCell::new(util::make_empty_bus_info());

        let mut gb = GameBoyPlugin::allocate(gbi, audio_out, event_in);

        gb.init_event_in();
        gb.init_audio_out();

        gb
    }

    pub fn bus_count(&self, media_type: MediaTypes, dir: BusDirections) -> i32 {
        match media_type {
            MediaTypes::kAudio => match dir {
                BusDirections::kInput => 0,
                BusDirections::kOutput => 1,
            },
            MediaTypes::kEvent => match dir {
                BusDirections::kInput => 1,
                BusDirections::kOutput => 0,
            },
            MediaTypes::kNumMediaTypes => 0,
        }
    }
}

impl IPluginBase for GameBoyPlugin {
    unsafe fn initialize(&self, _host_context: *mut c_void) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}

impl IComponent for GameBoyPlugin {
    unsafe fn get_controller_class_id(&self, tuid: *mut IID) -> tresult {
        *tuid = GameBoyController::CID;

        kResultOk
    }

    unsafe fn set_io_mode(&self, _mode: i32) -> tresult {
        kResultOk
    }

    unsafe fn get_bus_count(&self, media_type: i32, dir: i32) -> i32 {
        if let Some(media_type) = util::as_media_type(media_type) {
            if let Some(dir) = util::as_bus_dir(dir) {
                return self.bus_count(media_type, dir);
            }
        }
        0
    }

    unsafe fn get_bus_info(
        &self,
        media_type: i32,
        dir: i32,
        idx: i32,
        info: *mut BusInfo,
    ) -> tresult {
        let info = &mut *info;

        match util::as_media_type(media_type) {
            Some(MediaTypes::kAudio) => match util::as_bus_dir(dir) {
                Some(BusDirections::kOutput) => {
                    if idx == 0 {
                        let bus = self.audio_out.borrow();

                        util::str128cpy(&bus.name, &mut info.name);
                        info.media_type = bus.media_type as i32;
                        info.direction = bus.direction as i32;
                        info.bus_type = bus.bus_type as i32;
                        info.channel_count = bus.channel_count;
                        info.flags = bus.flags as u32;

                        kResultOk
                    } else {
                        kInvalidArgument
                    }
                }
                _ => kInvalidArgument,
            },
            Some(MediaTypes::kEvent) => match util::as_bus_dir(dir) {
                Some(BusDirections::kInput) => {
                    if idx == 0 {
                        let bus = self.event_in.borrow();

                        util::str128cpy(&bus.name, &mut info.name);
                        info.media_type = bus.media_type as i32;
                        info.direction = bus.direction as i32;
                        info.bus_type = bus.bus_type as i32;
                        info.channel_count = bus.channel_count;
                        info.flags = bus.flags as u32;

                        kResultOk
                    } else {
                        kInvalidArgument
                    }
                }
                _ => kInvalidArgument,
            },
            _ => kInvalidArgument,
        }
    }

    unsafe fn get_routing_info(
        &self,
        _in_info: *mut RoutingInfo,
        _out_info: *mut RoutingInfo,
    ) -> i32 {
        kResultFalse
    }

    unsafe fn activate_bus(&self, _type_: i32, _dir: i32, _idx: i32, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn set_active(&self, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn set_state(&self, _state: *mut c_void) -> tresult {
        kResultOk
    }

    unsafe fn get_state(&self, _state: *mut c_void) -> tresult {
        kResultOk
    }
}

impl IAudioProcessor for GameBoyPlugin {
    unsafe fn set_bus_arrangements(
        &self,
        _inputs: *mut u64,
        _num_inputs: i32,
        _outputs: *mut u64,
        _num_outputs: i32,
    ) -> i32 {
        kResultOk
    }

    unsafe fn get_bus_arrangement(&self, _dir: i32, _idx: i32, arr: *mut u64) -> i32 {
        let arr = &mut *arr;
        if (*arr == 0x0) || (*arr == 0x1) || (*arr == 0x3) {
            kResultOk
        } else {
            *arr = 0x03;
            kResultOk
        }
    }

    unsafe fn can_process_sample_size(&self, symbolic_sample_size: i32) -> i32 {
        match symbolic_sample_size {
            K_SAMPLE32 => kResultTrue,
            K_SAMPLE64 => kResultTrue,
            _ => kResultFalse,
        }
    }

    unsafe fn get_latency_samples(&self) -> u32 {
        0
    }

    unsafe fn setup_processing(&self, _setup: *const ProcessSetup) -> tresult {
        kResultOk
    }

    unsafe fn set_processing(&self, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn get_tail_samples(&self) -> u32 {
        0
    }

    unsafe fn process(&self, data: *mut ProcessData) -> tresult {
        let data = &*data;

        if data.input_events.is_null() || data.outputs.is_null() {
            return kResultOk;
        }

        // process parameters
        if !data.input_param_changes.is_null() {
            let param_changes = data.input_param_changes.upgrade().unwrap();
            let count = param_changes.get_parameter_count();

            for i in 0..count {
                let param_queue = param_changes.get_parameter_data(i);
                if let Some(param_queue) = param_queue.upgrade() {
                    let mut value = 0.0;
                    let mut sample_offset = 0;
                    let num_points = param_queue.get_point_count();
                    match Parameter::try_from(param_queue.get_parameter_id()) {
                        Ok(Parameter::MasterVolume) => {
                            if param_queue.get_point(
                                num_points - 1,
                                &mut sample_offset as *mut _,
                                &mut value as *mut _,
                            ) == kResultTrue
                            {
                                self.gbi
                                    .borrow_mut()
                                    .set_param(&Parameter::MasterVolume, value);
                            }
                        }
                        // ここすっきりさせたいなー
                        Ok(Parameter::AttackTime) => {
                            if param_queue.get_point(
                                num_points - 1,
                                &mut sample_offset as *mut _,
                                &mut value as *mut _,
                            ) == kResultTrue
                            {
                                self.gbi
                                    .borrow_mut()
                                    .set_param(&Parameter::AttackTime, value);
                            }
                        }
                        Ok(Parameter::DecayTime) => {
                            if param_queue.get_point(
                                num_points - 1,
                                &mut sample_offset as *mut _,
                                &mut value as *mut _,
                            ) == kResultTrue
                            {
                                self.gbi
                                    .borrow_mut()
                                    .set_param(&Parameter::DecayTime, value);
                            }
                        }
                        Ok(Parameter::Sustain) => {
                            if param_queue.get_point(
                                num_points - 1,
                                &mut sample_offset as *mut _,
                                &mut value as *mut _,
                            ) == kResultTrue
                            {
                                self.gbi.borrow_mut().set_param(&Parameter::Sustain, value);
                            }
                        }
                        Ok(Parameter::ReleaseTime) => {
                            if param_queue.get_point(
                                num_points - 1,
                                &mut sample_offset as *mut _,
                                &mut value as *mut _,
                            ) == kResultTrue
                            {
                                self.gbi
                                    .borrow_mut()
                                    .set_param(&Parameter::ReleaseTime, value);
                            }
                        }
                        Err(_) => (),
                    }
                }
            }
        }

        // process event inputs
        if !data.input_events.is_null() {
            let input_events = data.input_events.upgrade().unwrap();
            let count = input_events.get_event_count();

            for c in 0..count {
                let mut e = util::make_empty_event();

                if input_events.get_event(c, &mut e) == kResultOk {
                    let mut gbi = self.gbi.borrow_mut();
                    match util::as_event_type(e.type_) {
                        Some(EventTypes::kNoteOnEvent) => gbi.note_on(e.event.note_on.pitch),
                        Some(EventTypes::kNoteOffEvent) => gbi.note_off(),
                        Some(_) => (),
                        _ => (),
                    }
                }
            }
        }

        // process audio outputs
        let num_samples = data.num_samples as usize;
        let outputs: &mut AudioBusBuffers = &mut *data.outputs;
        let num_output_channels = outputs.num_channels as usize;

        let sample_rate = (*(data.context)).sample_rate;

        let out = (*(*data).outputs).buffers;

        match data.symbolic_sample_size {
            K_SAMPLE32 => {
                for n in 0..num_samples as isize {
                    let s = self.gbi.borrow_mut().process(sample_rate);

                    for i in 0..num_output_channels as isize {
                        let ch_out = *out.offset(i) as *mut f32;
                        *ch_out.offset(n) = s.0 as f32;
                    }
                }

                kResultOk
            }
            K_SAMPLE64 => {
                for n in 0..num_samples as isize {
                    let s = self.gbi.borrow_mut().process(sample_rate);

                    for i in 0..num_output_channels as isize {
                        let ch_out = *out.offset(i) as *mut f64;
                        *ch_out.offset(n) = s.0;
                    }
                }

                kResultOk
            }
            _ => unreachable!(),
        }
    }
}
