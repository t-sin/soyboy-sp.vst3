use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};

use bincode::Options;

use vst3_com::{interfaces::IUnknown, ComInterface};
use vst3_sys::{
    base::kResultOk,
    utils::SharedVstPtr,
    vst::{IAttributeList, IConnectionPoint, IHostApplication, IMessage},
    VstPtr,
};

use super::raw_utils::fidstring_to_string;
use crate::common::{constants, i4, PluginConfigV02, Vst3Message, Waveform};

pub struct SyncPtr<I: ComInterface + ?Sized> {
    ptr: VstPtr<I>,
}

unsafe impl<I: ComInterface + ?Sized> Sync for SyncPtr<I> {}
unsafe impl<I: ComInterface + ?Sized> Send for SyncPtr<I> {}

impl<I: ComInterface + ?Sized> SyncPtr<I> {
    pub fn new(ptr: VstPtr<I>) -> Self {
        Self { ptr }
    }

    pub fn ptr(&self) -> &VstPtr<I> {
        &self.ptr
    }
}

pub struct ComPtr<I: ComInterface + ?Sized> {
    _ptr: *mut c_void,
    obj: VstPtr<I>,
}

impl<I: ComInterface + ?Sized> ComPtr<I> {
    pub fn new(_ptr: *mut c_void, obj: VstPtr<I>) -> Self {
        Self { _ptr, obj }
    }

    pub fn obj(&self) -> VstPtr<I> {
        self.obj.clone()
    }
}

impl<I: ComInterface + ?Sized> Drop for ComPtr<I> {
    fn drop(&mut self) {
        unsafe {
            self.obj.release();
        }
    }
}

pub struct ControllerConnection {
    conn: Arc<dyn IConnectionPoint>,
    host: Arc<ComPtr<dyn IHostApplication>>,
}

unsafe impl Sync for ControllerConnection {}
unsafe impl Send for ControllerConnection {}

impl ControllerConnection {
    pub fn new(conn: Arc<dyn IConnectionPoint>, host: Arc<ComPtr<dyn IHostApplication>>) -> Self {
        Self { conn, host }
    }

    pub fn send_message(&self, msg: Vst3Message) {
        let imsg = allocate_message(&self.host.obj());

        if let Some(mut imsg) = imsg {
            msg.write_message(&mut imsg);
            unsafe {
                let imsg = std::mem::transmute::<VstPtr<dyn IMessage>, SharedVstPtr<dyn IMessage>>(
                    imsg.obj(),
                );
                self.conn.notify(imsg);
            }

            unsafe { imsg.obj().release() };
        } else {
            log::error!("SoyBoyPlugin::send_message(): allocation failed");
        }
    }
}

impl Vst3Message {
    pub fn from_message(msg: &SharedVstPtr<dyn IMessage>) -> Option<Self> {
        let msg = msg.upgrade().unwrap();
        let id = unsafe { fidstring_to_string(msg.get_message_id()) };

        match id.as_str() {
            "vst3:note-on" => Some(Vst3Message::NoteOn),
            "vst3:initialize-wavetable" => Some(Vst3Message::InitializeWaveTable),
            "vst3:randomize-wavetable" => Some(Vst3Message::RandomizeWaveTable),
            "vst3:wavetable-data" => {
                let attr = unsafe { msg.get_attributes() };
                let attr_id = CString::new("table").unwrap();
                let mut size: u32 = 0;
                let table_ptr: *mut c_void = null_mut();

                unsafe {
                    attr.upgrade().unwrap().get_binary(
                        attr_id.as_ptr(),
                        &table_ptr as *const _,
                        &mut size as *mut _,
                    );
                };

                let table_ptr = table_ptr as *mut i8;
                let table_src = unsafe { std::slice::from_raw_parts(table_ptr, size as usize) };
                let mut table: [i4; constants::WAVETABLE_SIZE] =
                    [i4::from(0i8); constants::WAVETABLE_SIZE];
                for (i, v) in table_src.iter().enumerate() {
                    table[i] = i4::from(*v);
                }

                Some(Vst3Message::WaveTableData(table))
            }
            "vst3:config-requested" => Some(Vst3Message::ConfigurationRequested),
            "vst3:config-data" => {
                let attr = unsafe { msg.get_attributes() };
                let attr_id = CString::new("config").unwrap();

                let mut size: u32 = 0;
                let bytes_ptr: *mut c_void = null_mut();

                unsafe {
                    attr.upgrade().unwrap().get_binary(
                        attr_id.as_ptr(),
                        &bytes_ptr as *const _,
                        &mut size as *mut _,
                    );
                };

                let bytes_ptr = bytes_ptr as *mut u8;
                let bytes_src = unsafe { std::slice::from_raw_parts(bytes_ptr, size as usize) };

                let options = bincode::config::DefaultOptions::new()
                    .reject_trailing_bytes()
                    .with_little_endian()
                    .with_fixint_encoding();
                let decoded = options.deserialize(&bytes_src[..]);
                if decoded.is_err() {
                    log::error!("cannot decode configuration data");
                    return Some(Vst3Message::ConfigurationData(PluginConfigV02::default()));
                }

                let config: PluginConfigV02 = decoded.unwrap();
                Some(Vst3Message::ConfigurationData(config))
            }
            "vst3:set-wavetable-sample" => {
                let attr = unsafe { msg.get_attributes() };
                let id_idx = CString::new("index").unwrap();
                let id_val = CString::new("value").unwrap();
                let mut idx: i64 = 0;
                let mut val: i64 = 0;

                unsafe {
                    attr.upgrade()
                        .unwrap()
                        .get_int(id_idx.as_ptr(), &mut idx as *mut _);
                    attr.upgrade()
                        .unwrap()
                        .get_int(id_val.as_ptr(), &mut val as *mut _);
                };

                Some(Vst3Message::SetWaveTable(idx as usize, i4::from(val as i8)))
            }
            "vst3:waveform-data" => {
                let attr = unsafe { msg.get_attributes() };
                let attr_id = CString::new("signals").unwrap();
                let mut size: u32 = 0;
                let signals_ptr: *mut c_void = null_mut();

                unsafe {
                    attr.upgrade().unwrap().get_binary(
                        attr_id.as_ptr(),
                        &signals_ptr as *const _,
                        &mut size as *mut _,
                    );
                };

                let signals_ptr = signals_ptr as *mut f64;
                let signals_src = unsafe {
                    std::slice::from_raw_parts(signals_ptr, constants::OSCILLOSCOPE_SAIMPLE_SIZE)
                };
                let mut signals: [f64; constants::OSCILLOSCOPE_SAIMPLE_SIZE] =
                    [0.0; constants::OSCILLOSCOPE_SAIMPLE_SIZE];
                signals
                    .as_mut_slice()
                    .copy_from_slice(&signals_src[..constants::OSCILLOSCOPE_SAIMPLE_SIZE]);

                let mut wf = Waveform::new();
                wf.set_signals(&signals);

                Some(Vst3Message::WaveformData(wf))
            }
            "vst3:enable-waveform" => Some(Vst3Message::EnableWaveform),
            "vst3:disable-waveform" => Some(Vst3Message::DisableWaveform),
            _ => None,
        }
    }

    fn to_cstring(&self) -> CString {
        CString::new(self.to_string()).unwrap()
    }

    pub fn write_message(&self, msg: &mut ComPtr<dyn IMessage>) {
        let msg = msg.obj();

        match self {
            Vst3Message::NoteOn => {
                unsafe { msg.set_message_id(self.to_cstring().as_ptr()) };
            }
            Vst3Message::InitializeWaveTable => unsafe {
                msg.set_message_id(self.to_cstring().as_ptr());
            },
            Vst3Message::RandomizeWaveTable => {
                unsafe { msg.set_message_id(self.to_cstring().as_ptr()) };
            }
            Vst3Message::WaveTableData(i4table) => {
                unsafe { msg.set_message_id(self.to_cstring().as_ptr()) };

                let attr = unsafe { msg.get_attributes() };
                let attr_id = CString::new("table").unwrap();
                let mut table: [i8; constants::WAVETABLE_SIZE] = [0; constants::WAVETABLE_SIZE];
                let size = table.len() as u32;

                for (i, i4v) in i4table.iter().enumerate() {
                    table[i] = (*i4v).into();
                }

                unsafe {
                    attr.upgrade().unwrap().set_binary(
                        attr_id.as_ptr(),
                        table.as_ptr() as *const c_void,
                        size,
                    );
                };
            }
            Vst3Message::ConfigurationRequested => {
                unsafe { msg.set_message_id(self.to_cstring().as_ptr()) };
            }
            Vst3Message::ConfigurationData(config) => {
                unsafe { msg.set_message_id(self.to_cstring().as_ptr()) };
                let attr = unsafe { msg.get_attributes() };
                let attr_id = CString::new("config").unwrap();

                let options = bincode::config::DefaultOptions::new()
                    .reject_trailing_bytes()
                    .with_little_endian()
                    .with_fixint_encoding();
                let encoded = options.serialize(config);
                if encoded.is_err() {
                    log::error!("cannot encode configuration. it's a bug!");
                    return;
                }
                let bytes = encoded.unwrap();

                unsafe {
                    attr.upgrade().unwrap().set_binary(
                        attr_id.as_ptr(),
                        bytes.as_ptr() as *const c_void,
                        bytes.len() as u32,
                    );
                };
            }
            Vst3Message::SetWaveTable(idx, val) => {
                unsafe { msg.set_message_id(self.to_cstring().as_ptr()) };

                let attr = unsafe { msg.get_attributes() };
                let id_idx = CString::new("index").unwrap();
                let id_val = CString::new("value").unwrap();
                let val: i8 = (*val).into();
                let val = val as i64;

                unsafe {
                    attr.upgrade()
                        .unwrap()
                        .set_int(id_idx.as_ptr(), *idx as i64);
                    attr.upgrade().unwrap().set_int(id_val.as_ptr(), val);
                };
            }
            Vst3Message::WaveformData(wf) => {
                unsafe { msg.set_message_id(self.to_cstring().as_ptr()) };

                let attr = unsafe { msg.get_attributes() };
                let attr_id = CString::new("signals").unwrap();

                let signals = wf.get_signals();
                let size = signals.len() * std::mem::size_of::<f64>();

                unsafe {
                    attr.upgrade().unwrap().set_binary(
                        attr_id.as_ptr(),
                        signals.as_ptr() as *const c_void,
                        size as u32,
                    );
                };
            }
            Vst3Message::EnableWaveform => {
                unsafe { msg.set_message_id(self.to_cstring().as_ptr()) };
            }
            Vst3Message::DisableWaveform => {
                unsafe { msg.set_message_id(self.to_cstring().as_ptr()) };
            }
        }
    }
}

pub fn allocate_message(host: &VstPtr<dyn IHostApplication>) -> Option<ComPtr<dyn IMessage>> {
    let iid = <dyn IMessage as ComInterface>::IID;
    let iid = &iid as *const _;
    let mut msg_ptr: *mut c_void = null_mut();

    let result = unsafe { host.create_instance(iid, iid, &mut msg_ptr as *mut _) };
    if result != kResultOk {
        #[cfg(debug_assertions)]
        print!(
            "Vst3Message::allocate(): calling IHostApplication::create_instance() failed because "
        );

        return None;
    }

    let msg_obj = unsafe { VstPtr::shared(msg_ptr as *mut _).unwrap() };

    Some(ComPtr::new(msg_ptr, msg_obj))
}

pub fn get_host_app(context: &VstPtr<dyn IUnknown>) -> ComPtr<dyn IHostApplication> {
    let host_iid = <dyn IHostApplication as ComInterface>::IID;
    let mut host_ptr: *mut c_void = null_mut();

    let result = unsafe { context.query_interface(&host_iid as *const _, &mut host_ptr as *mut _) };

    if result != kResultOk {
        panic!("host context is not implemented IHostApplication");
    }

    let host_obj = unsafe { VstPtr::shared(host_ptr as *mut _).unwrap() };

    ComPtr::new(host_ptr, host_obj)
}

pub fn send_message(
    connection: Arc<Mutex<SyncPtr<dyn IConnectionPoint>>>,
    msg: &ComPtr<dyn IMessage>,
) {
    let connection = connection.lock().unwrap();
    let connection = connection.ptr();

    unsafe {
        let msg =
            std::mem::transmute::<VstPtr<dyn IMessage>, SharedVstPtr<dyn IMessage>>(msg.obj());
        connection.notify(msg);
    }
}

macro_rules! read_config {
    ($config:ident, $state:ident) => {
        let options = bincode::config::DefaultOptions::new()
            .reject_trailing_bytes()
            .with_little_endian()
            .with_fixint_encoding();
        let size = options.serialized_size(&$config).unwrap();
        let mut bytes: Vec<u8> = vec![0; size as usize];

        let result = $state.read(bytes.as_mut_ptr() as *mut c_void, size as i32, null_mut());

        if result != kResultOk {
            log::error!("read_config!: cannot read PluginConfig");
            return kResultFalse;
        }

        let decoded = options.deserialize(&bytes[..]);
        if decoded.is_err() {
            log::error!("read_config!: invalid config: {:?}", decoded);
            return kResultFalse;
        }

        $config = decoded.unwrap();
    };
}

macro_rules! write_config {
    ($version:expr, $config:expr, $state:ident) => {
        let options = bincode::config::DefaultOptions::new()
            .reject_trailing_bytes()
            .with_little_endian()
            .with_fixint_encoding();

        let encoded = options.serialize($config);
        if encoded.is_err() {
            log::error!("cannot encode configuration. it's a bug!");
            return kResultFalse;
        }
        let bytes = encoded.unwrap();

        let result = $state.write(
            &$version as *const _ as *const c_void,
            mem::size_of::<u32>() as i32,
            null_mut(),
        );

        if result != kResultOk {
            log::error!("cannot write CONFIG_VERSION");
            return kResultFalse;
        }

        let result = $state.write(
            bytes.as_ptr() as *const c_void,
            bytes.len() as i32,
            null_mut(),
        );

        if result != kResultOk {
            log::error!("cannot write PluginConfigV{:02}", $version);
            return kResultFalse;
        }
    };
}

pub(crate) use read_config;
pub(crate) use write_config;
