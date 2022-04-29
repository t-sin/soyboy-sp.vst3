use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::mem;
use std::os::raw::c_void;
use std::sync::{Arc, Mutex};

use vst3_com::{interfaces::IUnknown, sys::GUID, IID};
use vst3_sys::{
    base::{
        kInvalidArgument, kResultFalse, kResultOk, kResultTrue, tresult, IBStream, IPluginBase,
        TBool,
    },
    utils::SharedVstPtr,
    vst::{
        AudioBusBuffers, BusDirections, BusFlags, BusInfo, BusTypes, EventTypes, IAudioProcessor,
        IComponent, IConnectionPoint, IEventList, IMessage, IParamValueQueue, IParameterChanges,
        MediaTypes, ProcessData, ProcessSetup, RoutingInfo, K_SAMPLE32, K_SAMPLE64,
    },
    VstPtr, VST3,
};

use crate::common::Waveform;
use crate::soyboy::{
    event::{Event, Triggered},
    parameters::{Normalizable, ParameterDef, Parametric, SoyBoyParameter},
    AudioProcessor, SoyBoy,
};
use crate::vst3::{controller::SoyBoyController, message::Vst3Message, plugin_data, utils};

#[VST3(implements(IComponent, IAudioProcessor, IConnectionPoint))]
pub struct SoyBoyPlugin {
    soyboy: Mutex<SoyBoy>,
    param_defs: HashMap<SoyBoyParameter, ParameterDef>,
    audio_out: RefCell<BusInfo>,
    event_in: RefCell<BusInfo>,
    context: Mutex<Option<VstPtr<dyn IUnknown>>>,
    controller: Mutex<Option<VstPtr<dyn IConnectionPoint>>>,
    waveform: Arc<Mutex<Waveform>>,
    elapsed_samples: RefCell<u32>,
}

impl SoyBoyPlugin {
    pub const CID: GUID = GUID {
        data: plugin_data::VST3_CID,
    };

    unsafe fn init_event_in(&self) {
        let mut bus = self.event_in.borrow_mut();

        utils::wstrcpy("Event In", bus.name.as_mut_ptr());
        bus.media_type = MediaTypes::kEvent as i32;
        bus.direction = BusDirections::kInput as i32;
        bus.channel_count = 1;
        bus.bus_type = BusTypes::kMain as i32;
        bus.flags = BusFlags::kDefaultActive as u32;
    }

    unsafe fn init_audio_out(&self) {
        let mut bus = self.audio_out.borrow_mut();

        utils::wstrcpy("Audio Out", bus.name.as_mut_ptr());
        bus.media_type = MediaTypes::kAudio as i32;
        bus.direction = BusDirections::kOutput as i32;
        bus.channel_count = 2;
        bus.bus_type = BusTypes::kMain as i32;
        bus.flags = BusFlags::kDefaultActive as u32;
    }

    pub unsafe fn new(param_defs: HashMap<SoyBoyParameter, ParameterDef>) -> Box<Self> {
        let soyboy = Mutex::new(SoyBoy::new());
        let audio_out = RefCell::new(utils::make_empty_bus_info());
        let event_in = RefCell::new(utils::make_empty_bus_info());
        let controller = Mutex::new(None);
        let context = Mutex::new(None);
        let waveform = Arc::new(Mutex::new(Waveform::new()));
        let elapsed_samples = RefCell::new(0);

        SoyBoyPlugin::allocate(
            soyboy,
            param_defs,
            audio_out,
            event_in,
            context,
            controller,
            waveform,
            elapsed_samples,
        )
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

    fn send_message(&self, msg: Vst3Message) {
        if let Some(controller) = &*self.controller.lock().unwrap() {
            if let Some(context) = &*self.context.lock().unwrap() {
                let host = utils::get_host_app(context).obj();

                let msg = msg.allocate(&host);
                if let Some(msg) = msg {
                    unsafe {
                        let msg = std::mem::transmute::<
                            VstPtr<dyn IMessage>,
                            SharedVstPtr<dyn IMessage>,
                        >(msg.obj());
                        controller.notify(msg);
                    }
                } else {
                    println!("SoyBoyPlugin::send_message(): allocation failed");
                }
            }
        }
    }
}

impl IPluginBase for SoyBoyPlugin {
    unsafe fn initialize(&self, host_context: *mut c_void) -> tresult {
        if host_context.is_null() {
            panic!("host context is null");
        }

        let context: VstPtr<dyn IUnknown> = VstPtr::shared(host_context as *mut _).unwrap();
        *self.context.lock().unwrap() = Some(context);

        for param in SoyBoyParameter::iter() {
            if let Some(sp) = self.param_defs.get(&param) {
                self.soyboy
                    .lock()
                    .unwrap()
                    .set_param(&param, sp.default_value);
            }
        }

        self.init_event_in();
        self.init_audio_out();

        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        #[cfg(debug_assertions)]
        println!("SoyBoyPlugin::terminate()");

        kResultOk
    }
}

impl IComponent for SoyBoyPlugin {
    unsafe fn get_controller_class_id(&self, tuid: *mut IID) -> tresult {
        *tuid = SoyBoyController::CID;

        kResultOk
    }

    unsafe fn set_io_mode(&self, _mode: i32) -> tresult {
        kResultOk
    }

    unsafe fn get_bus_count(&self, media_type: i32, dir: i32) -> i32 {
        if let Some(media_type) = utils::as_media_type(media_type) {
            if let Some(dir) = utils::as_bus_dir(dir) {
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

        match utils::as_media_type(media_type) {
            Some(MediaTypes::kAudio) => match utils::as_bus_dir(dir) {
                Some(BusDirections::kOutput) => {
                    if idx == 0 {
                        let bus = self.audio_out.borrow();

                        utils::str128cpy(&bus.name, &mut info.name);
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
            Some(MediaTypes::kEvent) => match utils::as_bus_dir(dir) {
                Some(BusDirections::kInput) => {
                    if idx == 0 {
                        let bus = self.event_in.borrow();

                        utils::str128cpy(&bus.name, &mut info.name);
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

    unsafe fn set_state(&self, state: SharedVstPtr<dyn IBStream>) -> tresult {
        if state.is_null() {
            return kResultFalse;
        }

        let state = state.upgrade();
        if state.is_none() {
            return kResultFalse;
        }
        let state = state.unwrap();

        let mut num_bytes_read = 0;
        for param in SoyBoyParameter::iter() {
            if let Some(p) = self.param_defs.get(&param) {
                let mut value = 0.0;
                let ptr = &mut value as *mut f64 as *mut c_void;

                state.read(ptr, mem::size_of::<f64>() as i32, &mut num_bytes_read);

                self.soyboy
                    .lock()
                    .unwrap()
                    .set_param(&param, p.denormalize(value));
            } else {
                return kResultFalse;
            }
        }

        kResultOk
    }

    unsafe fn get_state(&self, state: SharedVstPtr<dyn IBStream>) -> tresult {
        if state.is_null() {
            return kResultFalse;
        }

        let state = state.upgrade();
        if state.is_none() {
            return kResultFalse;
        }
        let state = state.unwrap();

        let mut num_bytes_written = 0;
        for param in SoyBoyParameter::iter() {
            if let Some(p) = self.param_defs.get(&param) {
                let value = self.soyboy.lock().unwrap().get_param(&param);

                let mut value = p.normalize(value);
                let ptr = &mut value as *mut f64 as *mut c_void;
                state.write(ptr, mem::size_of::<f64>() as i32, &mut num_bytes_written);
            } else {
                return kResultFalse;
            }
        }

        kResultOk
    }
}

impl IAudioProcessor for SoyBoyPlugin {
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
        let mut soyboy = self.soyboy.lock().unwrap();

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
                    if let Ok(param) = SoyBoyParameter::try_from(param_queue.get_parameter_id()) {
                        if param_queue.get_point(
                            num_points - 1,
                            &mut sample_offset as *mut _,
                            &mut value as *mut _,
                        ) == kResultTrue
                        {
                            if let Some(p) = self.param_defs.get(&param) {
                                soyboy.set_param(&param, p.denormalize(value));
                            }
                        }
                    }
                }
            }
        }

        // process event inputs
        if !data.input_events.is_null() {
            let input_events = data.input_events.upgrade().unwrap();
            let count = input_events.get_event_count();

            for c in 0..count {
                let mut e = utils::make_empty_event();

                if input_events.get_event(c, &mut e) == kResultOk {
                    match utils::as_event_type(e.type_) {
                        Some(EventTypes::kNoteOnEvent) => {
                            self.send_message(Vst3Message::NoteOn);
                            soyboy.trigger(&Event::NoteOn {
                                note: e.event.note_on.pitch as u16,
                                velocity: e.event.note_on.velocity as f64,
                            });
                        }
                        Some(EventTypes::kNoteOffEvent) => {
                            soyboy.trigger(&Event::NoteOff {
                                note: e.event.note_off.pitch as u16,
                            });
                        }
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

        let mut waveform = self.waveform.lock().unwrap();

        match data.symbolic_sample_size {
            K_SAMPLE32 => {
                for n in 0..num_samples as isize {
                    let s = soyboy.process(sample_rate);
                    waveform.set_signal(s.0);

                    for i in 0..num_output_channels as isize {
                        let ch_out = *out.offset(i) as *mut f32;
                        *ch_out.offset(n) = s.0 as f32;
                    }
                }
            }
            K_SAMPLE64 => {
                for n in 0..num_samples as isize {
                    let s = soyboy.process(sample_rate);
                    waveform.set_signal(s.0);

                    for i in 0..num_output_channels as isize {
                        let ch_out = *out.offset(i) as *mut f64;
                        *ch_out.offset(n) = s.0;
                    }
                }
            }
            _ => unreachable!(),
        }

        {
            *self.elapsed_samples.borrow_mut() += num_samples as u32;
        }
        let elapsed = *self.elapsed_samples.borrow();
        let samples_a_frame = (sample_rate / 30.0) as u32;
        if elapsed > samples_a_frame {
            *self.elapsed_samples.borrow_mut() = 0;
            let wf = self.waveform.borrow_mut().clone();
            self.send_message(Vst3Message::WaveformData(wf));
        }

        kResultOk
    }
}

impl IConnectionPoint for SoyBoyPlugin {
    unsafe fn connect(&self, other: SharedVstPtr<dyn IConnectionPoint>) -> tresult {
        #[cfg(debug_assertions)]
        println!("IConnectionPoint::connect() on SoyBoyPlugin");

        let other = other.upgrade().unwrap();
        *self.controller.lock().unwrap() = Some(other);
        #[cfg(debug_assertions)]
        println!("IConnectionPoint::connect() on SoyBoyPlugin: connected");

        kResultOk
    }

    unsafe fn disconnect(&self, _other: SharedVstPtr<dyn IConnectionPoint>) -> tresult {
        #[cfg(debug_assertions)]
        println!("IConnectionPoint::disconnect() on SoyBoyPlugin");

        *self.controller.lock().unwrap() = None;
        kResultOk
    }

    unsafe fn notify(&self, message: SharedVstPtr<dyn IMessage>) -> tresult {
        #[cfg(debug_assertions)]
        println!("IConnectionPoint::notify() on SoyBoyPlugin");

        match Vst3Message::from_message(&message) {
            Some(Vst3Message::InitializeWaveTable) => {
                let mut soyboy = self.soyboy.lock().unwrap();
                soyboy.trigger(&Event::ResetWaveTableAsSine);
                let table = soyboy.get_wavetable();
                self.send_message(Vst3Message::WaveTableData(table));
            }
            Some(Vst3Message::RandomizeWaveTable) => {
                let mut soyboy = self.soyboy.lock().unwrap();

                soyboy.trigger(&Event::ResetWaveTableAtRandom);
                let table = soyboy.get_wavetable();
                self.send_message(Vst3Message::WaveTableData(table));
            }
            Some(Vst3Message::WaveTableRequested) => {
                let table = self.soyboy.lock().unwrap().get_wavetable();
                self.send_message(Vst3Message::WaveTableData(table));
            }
            Some(Vst3Message::SetWaveTable(idx, value)) => {
                let mut soyboy = self.soyboy.lock().unwrap();

                soyboy.trigger(&Event::SetWaveTable { idx, value });
                let table = soyboy.get_wavetable();
                self.send_message(Vst3Message::WaveTableData(table));
            }
            _ => (),
        }
        kResultOk
    }
}
