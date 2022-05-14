use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::convert::TryFrom;
use std::mem;
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

use bincode::Options;

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

use crate::common::{constants, PluginConfigV01, Vst3Message, Waveform};
use crate::soyboy::{
    event::{Event, Triggered},
    parameters::{Normalizable, ParameterDef, Parametric, SoyBoyParameter},
    AudioProcessor, SoyBoy,
};
use crate::vst3::{
    controller::SoyBoyController, plugin_data, raw_utils, vst3_utils, vst3_utils::SyncPtr,
};

pub struct PluginTimerThread {
    handle: RefCell<Option<thread::JoinHandle<()>>>,
    quit: Arc<Mutex<bool>>,
}

impl PluginTimerThread {
    fn new() -> Self {
        Self {
            handle: RefCell::new(None),
            quit: Arc::new(Mutex::new(false)),
        }
    }

    fn start_thread(
        &mut self,
        config: Arc<Mutex<PluginConfigV01>>,
        host_context: Arc<Mutex<SyncPtr<dyn IUnknown>>>,
        controller: Arc<Mutex<SyncPtr<dyn IConnectionPoint>>>,
        waveform: Arc<Mutex<Waveform>>,
        queue: Arc<Mutex<VecDeque<Vst3Message>>>,
    ) {
        let config = config.clone();
        let context = host_context.clone();
        let connection = controller.clone();
        let waveform = waveform.clone();
        let queue = queue.clone();
        let quit = self.quit.clone();

        let handle = thread::spawn(move || {
            let host = {
                let context = context.lock().unwrap();
                let context = context.ptr();
                vst3_utils::get_host_app(&context).obj()
            };

            let mut msg_note_on = vst3_utils::allocate_message(&host).unwrap();
            let mut msg_waveform = vst3_utils::allocate_message(&host).unwrap();

            loop {
                if *quit.lock().unwrap() {
                    break;
                }

                {
                    let mut queue = queue.lock().unwrap();
                    loop {
                        match queue.pop_front() {
                            Some(msg) => {
                                msg.write_message(&mut msg_note_on);
                                vst3_utils::send_message(connection.clone(), &msg_note_on);
                            }
                            None => break,
                        }
                    }
                }

                if config.lock().unwrap().waveform_view_enabled {
                    let wf = waveform.lock().unwrap().clone();
                    Vst3Message::WaveformData(wf).write_message(&mut msg_waveform);
                    vst3_utils::send_message(connection.clone(), &msg_waveform);

                    thread::sleep(time::Duration::from_millis(
                        constants::WAVEFORM_UPDATE_INTERVAL_IN_MILLIS,
                    ));
                } else {
                    thread::sleep(time::Duration::from_millis(
                        constants::NORMAL_REDRAW_INTERVAL_IN_MILLIS,
                    ));
                }
            }
        });

        self.handle.replace(Some(handle));
    }

    fn stop_thread(&mut self) {
        *self.quit.lock().unwrap() = true;

        if let Some(handle) = self.handle.replace(None) {
            let res = handle.join();
            println!("stop_thread(): res = {:?}", res);
        }
    }
}

#[VST3(implements(IComponent, IAudioProcessor, IConnectionPoint))]
pub struct SoyBoyPlugin {
    soyboy: Mutex<SoyBoy>,
    config: Arc<Mutex<PluginConfigV01>>,
    param_defs: HashMap<SoyBoyParameter, ParameterDef>,
    audio_out: RefCell<BusInfo>,
    event_in: RefCell<BusInfo>,
    context: RefCell<Option<Arc<Mutex<SyncPtr<dyn IUnknown>>>>>,
    controller: RefCell<Option<Arc<Mutex<SyncPtr<dyn IConnectionPoint>>>>>,
    waveform: Arc<Mutex<Waveform>>,
    event_queue: Arc<Mutex<VecDeque<Vst3Message>>>,
    timer_thread: RefCell<PluginTimerThread>,
}

impl SoyBoyPlugin {
    pub const CID: GUID = GUID {
        data: plugin_data::VST3_CID,
    };

    unsafe fn init_event_in(&self) {
        let mut bus = self.event_in.borrow_mut();

        raw_utils::wstrcpy("Event In", bus.name.as_mut_ptr());
        bus.media_type = MediaTypes::kEvent as i32;
        bus.direction = BusDirections::kInput as i32;
        bus.channel_count = 1;
        bus.bus_type = BusTypes::kMain as i32;
        bus.flags = BusFlags::kDefaultActive as u32;
    }

    unsafe fn init_audio_out(&self) {
        let mut bus = self.audio_out.borrow_mut();

        raw_utils::wstrcpy("Audio Out", bus.name.as_mut_ptr());
        bus.media_type = MediaTypes::kAudio as i32;
        bus.direction = BusDirections::kOutput as i32;
        bus.channel_count = 2;
        bus.bus_type = BusTypes::kMain as i32;
        bus.flags = BusFlags::kDefaultActive as u32;
    }

    pub unsafe fn new(param_defs: HashMap<SoyBoyParameter, ParameterDef>) -> Box<Self> {
        let soyboy = Mutex::new(SoyBoy::new());
        let config = Arc::new(Mutex::new(PluginConfigV01::default()));
        let audio_out = RefCell::new(raw_utils::make_empty_bus_info());
        let event_in = RefCell::new(raw_utils::make_empty_bus_info());
        let context = RefCell::new(None);
        let controller = RefCell::new(None);
        let waveform = Arc::new(Mutex::new(Waveform::new()));
        let event_queue = Arc::new(Mutex::new(VecDeque::new()));
        let timer_thread = RefCell::new(PluginTimerThread::new());

        SoyBoyPlugin::allocate(
            soyboy,
            config,
            param_defs,
            audio_out,
            event_in,
            context,
            controller,
            waveform,
            event_queue,
            timer_thread,
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
        if let Some(context) = self.context.borrow_mut().clone() {
            if let Some(controller) = self.controller.borrow_mut().clone() {
                let context = context.lock().unwrap();
                let context = context.ptr();
                let host = vst3_utils::get_host_app(&context).obj();
                let mut imsg = vst3_utils::allocate_message(&host).unwrap();
                msg.write_message(&mut imsg);

                vst3_utils::send_message(controller, &imsg);

                unsafe { imsg.obj().release() };
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
        let context = SyncPtr::new(context);
        self.context.replace(Some(Arc::new(Mutex::new(context))));

        {
            let mut soyboy = self.soyboy.lock().unwrap();
            let mut config = self.config.lock().unwrap();

            for param in SoyBoyParameter::iter() {
                if let Some(sp) = self.param_defs.get(&param) {
                    soyboy.set_param(&param, sp.default_value);
                    config.set_param(&param, sp.default_value);
                }
            }
        }

        self.init_event_in();
        self.init_audio_out();

        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        #[cfg(debug_assertions)]
        println!("SoyBoyPlugin::terminate()");

        self.timer_thread.borrow_mut().stop_thread();

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
        if let Some(media_type) = raw_utils::as_media_type(media_type) {
            if let Some(dir) = raw_utils::as_bus_dir(dir) {
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

        match raw_utils::as_media_type(media_type) {
            Some(MediaTypes::kAudio) => match raw_utils::as_bus_dir(dir) {
                Some(BusDirections::kOutput) => {
                    if idx == 0 {
                        let bus = self.audio_out.borrow();

                        raw_utils::str128cpy(&bus.name, &mut info.name);
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
            Some(MediaTypes::kEvent) => match raw_utils::as_bus_dir(dir) {
                Some(BusDirections::kInput) => {
                    if idx == 0 {
                        let bus = self.event_in.borrow();

                        raw_utils::str128cpy(&bus.name, &mut info.name);
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

        let mut config_version: u32 = 0;
        let result = state.read(
            &mut config_version as *mut u32 as *mut c_void,
            mem::size_of::<u32>() as i32,
            null_mut(),
        );

        if result != kResultOk {
            println!("IAudioProcessor::set_state(): read CONFIG_VERSION failed");
            return kResultFalse;
        }

        match config_version {
            PluginConfigV01::CONFIG_VERSION => {
                let options = bincode::config::DefaultOptions::new()
                    .reject_trailing_bytes()
                    .with_little_endian()
                    .with_fixint_encoding();
                let size = options
                    .serialized_size(&PluginConfigV01::default())
                    .unwrap();
                let mut bytes = vec![0; size as usize];

                let result = state.read(bytes.as_mut_ptr() as *mut c_void, size as i32, null_mut());
                if result != kResultOk {
                    println!("set_state(): state.read() fails with error code {}", result);
                    return kResultFalse;
                }

                let decoded = options.deserialize(&bytes[..]);
                if decoded.is_err() {
                    return kResultFalse;
                }

                let config: PluginConfigV01 = decoded.unwrap();
                let mut soyboy = self.soyboy.lock().unwrap();
                for param in SoyBoyParameter::iter() {
                    let value = config.get_param(&param);
                    soyboy.set_param(&param, value);
                }
                *self.config.lock().unwrap() = config;
            }
            _ => {
                println!("IAudioProcessor::set_state(): unsupported VST3 state");
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

        let options = bincode::config::DefaultOptions::new()
            .reject_trailing_bytes()
            .with_little_endian()
            .with_fixint_encoding();

        let config_version = PluginConfigV01::CONFIG_VERSION;
        let config = self.config.lock().unwrap();
        let encoded = options.serialize(&*config);
        if encoded.is_err() {
            println!("cannot encode configuration. it's a bug!");
            return kResultFalse;
        }
        let bytes = encoded.unwrap();

        let result = state.write(
            &config_version as *const _ as *const c_void,
            mem::size_of::<u32>() as i32,
            null_mut(),
        );

        if result != kResultOk {
            println!("cannot write CONFIG_VERSION");
            return kResultFalse;
        }

        state.write(
            bytes.as_ptr() as *const c_void,
            bytes.len() as i32,
            null_mut(),
        );

        if result != kResultOk {
            println!("cannot write PluginConfigV01");
            return kResultFalse;
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
        if let Some(context) = &*self.context.borrow_mut() {
            if let Some(controller) = &*self.controller.borrow_mut() {
                self.timer_thread.borrow_mut().start_thread(
                    self.config.clone(),
                    context.clone(),
                    controller.clone(),
                    self.waveform.clone(),
                    self.event_queue.clone(),
                );
            }
        }

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

            let mut config = self.config.lock().unwrap();

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
                                config.set_param(&param, p.denormalize(value));
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
                let mut e = raw_utils::make_empty_event();

                if input_events.get_event(c, &mut e) == kResultOk {
                    match raw_utils::as_event_type(e.type_) {
                        Some(EventTypes::kNoteOnEvent) => {
                            self.event_queue
                                .lock()
                                .unwrap()
                                .push_back(Vst3Message::NoteOn);
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

        kResultOk
    }
}

impl IConnectionPoint for SoyBoyPlugin {
    unsafe fn connect(&self, other: SharedVstPtr<dyn IConnectionPoint>) -> tresult {
        #[cfg(debug_assertions)]
        println!("IConnectionPoint::connect() on SoyBoyPlugin");

        let other = other.upgrade().unwrap();
        let other = SyncPtr::new(other);
        self.controller.replace(Some(Arc::new(Mutex::new(other))));
        #[cfg(debug_assertions)]
        println!("IConnectionPoint::connect() on SoyBoyPlugin: connected");

        kResultOk
    }

    unsafe fn disconnect(&self, _other: SharedVstPtr<dyn IConnectionPoint>) -> tresult {
        #[cfg(debug_assertions)]
        println!("IConnectionPoint::disconnect() on SoyBoyPlugin");

        self.controller.replace(None);
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
            Some(Vst3Message::EnableWaveform) => {
                (*self.config.lock().unwrap()).waveform_view_enabled = true;
            }
            Some(Vst3Message::DisableWaveform) => {
                (*self.config.lock().unwrap()).waveform_view_enabled = false;
            }
            _ => (),
        }
        kResultOk
    }
}
