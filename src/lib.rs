extern crate env_logger;
extern crate vst3_com;
extern crate vst3_sys;

mod plugin;
mod util;

use log::*;
use std::os::raw::c_void;

use vst3_com::sys::GUID;
use vst3_sys::{
    base::{kInvalidArgument, kResultFalse, kResultOk, IPluginFactory, PClassInfo, PFactoryInfo},
    VST3,
};

use crate::util::strcpy;

use crate::plugin::GameBoyPlugin;

#[VST3(implements(IPluginFactory))]
pub struct GameBoyPluginFactory {}

impl GameBoyPluginFactory {
    fn new() -> Box<Self> {
        Self::allocate()
    }
}

impl IPluginFactory for GameBoyPluginFactory {
    unsafe fn get_factory_info(&self, info: *mut PFactoryInfo) -> i32 {
        let info = &mut *info;

        // set information
        strcpy("t-sin", info.vendor.as_mut_ptr());
        strcpy("https://github.com/t-sin/gbi", info.url.as_mut_ptr());

        kResultOk
    }

    unsafe fn count_classes(&self) -> i32 {
        1
    }

    unsafe fn get_class_info(&self, idx: i32, info: *mut PClassInfo) -> i32 {
        match idx {
            0 => {
                let info = &mut *info;

                info.cardinality = 0x7FFF_FFFF;
                info.cid = GameBoyPlugin::CID;

                strcpy("Audio Module Class", info.category.as_mut_ptr());
                strcpy("gbi", info.name.as_mut_ptr());
            }
            _ => {
                return kInvalidArgument;
            }
        }

        kResultOk
    }

    unsafe fn create_instance(
        &self,
        cid: *const GUID,
        _riid: *const GUID,
        obj: *mut *mut core::ffi::c_void,
    ) -> i32 {
        let iid = *cid;
        match iid {
            GameBoyPlugin::CID => {
                let ptr = Box::into_raw(GameBoyPlugin::new()) as *mut c_void;
                *obj = ptr;
                kResultOk
            }
            _ => kResultFalse,
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn GetPluginFactory() -> *mut c_void {
    Box::into_raw(GameBoyPluginFactory::new()) as *mut c_void
}

pub fn init() {
    env_logger::init();
    info!("plugin library loaded");
}

#[cfg(target_os = "linux")]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn ModuleEntry(_: *mut c_void) -> bool {
    init();
    true
}
