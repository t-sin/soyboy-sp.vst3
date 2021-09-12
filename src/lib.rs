extern crate env_logger;
extern crate vst3_com;
extern crate vst3_sys;

mod constant;
mod plugin;
mod util;

use log::*;
use std::os::raw::c_void;

use vst3_com::sys::GUID;
use vst3_sys::{
    base::{
        kInvalidArgument, kResultFalse, kResultOk, tresult, IPluginFactory, IPluginFactory2,
        IPluginFactory3, PClassInfo, PClassInfo2, PClassInfoW, PFactoryInfo,
    },
    VST3,
};

use crate::util::{strcpy, wstrcpy};

use crate::plugin::GameBoyPlugin;

#[VST3(implements(IPluginFactory3))]
pub struct GameBoyPluginFactory {}

impl GameBoyPluginFactory {
    fn new() -> Box<Self> {
        Self::allocate()
    }
}

impl IPluginFactory for GameBoyPluginFactory {
    unsafe fn get_factory_info(&self, info: *mut PFactoryInfo) -> tresult {
        let info = &mut *info;

        // set information
        strcpy(constant::PLUGIN_VENDOR, info.vendor.as_mut_ptr());
        strcpy(constant::PLUGIN_URL, info.url.as_mut_ptr());
        strcpy(constant::PLUGIN_EMAIL, info.email.as_mut_ptr());

        kResultOk
    }

    unsafe fn count_classes(&self) -> i32 {
        1
    }

    unsafe fn get_class_info(&self, idx: i32, info: *mut PClassInfo) -> tresult {
        match idx {
            0 => {
                let info = &mut *info;

                info.cardinality = 0x7FFF_FFFF;
                info.cid = GameBoyPlugin::CID;

                strcpy(constant::PLUGIN_CLASS_NAME, info.name.as_mut_ptr());
                strcpy(constant::PLUGIN_CLASS_CATEGORY, info.category.as_mut_ptr());
                strcpy(constant::PLUGIN_CLASS_NAME, info.name.as_mut_ptr());
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

impl IPluginFactory2 for GameBoyPluginFactory {
    unsafe fn get_class_info2(&self, idx: i32, info: *mut PClassInfo2) -> tresult {
        match idx {
            0 => {
                let info = &mut *info;

                strcpy(constant::PLUGIN_CLASS_NAME, info.name.as_mut_ptr());
                strcpy(constant::PLUGIN_VENDOR, info.vendor.as_mut_ptr());
                strcpy(constant::PLUGIN_CLASS_VERSION, info.version.as_mut_ptr());
                strcpy(constant::PLUGIN_CLASS_CATEGORY, info.category.as_mut_ptr());
                strcpy(
                    constant::PLUGIN_CLASS_SUBCATEGORIES,
                    info.subcategories.as_mut_ptr(),
                );

                kResultOk
            }
            _ => return kInvalidArgument,
        }
    }
}

impl IPluginFactory3 for GameBoyPluginFactory {
    unsafe fn get_class_info_unicode(&self, idx: i32, info: *mut PClassInfoW) -> tresult {
        match idx {
            0 => {
                let info = &mut *info;

                wstrcpy(constant::PLUGIN_CLASS_NAME, info.name.as_mut_ptr());
                wstrcpy(constant::PLUGIN_VENDOR, info.vendor.as_mut_ptr());
                wstrcpy(constant::PLUGIN_CLASS_VERSION, info.version.as_mut_ptr());
                strcpy(constant::PLUGIN_CLASS_CATEGORY, info.category.as_mut_ptr());
                strcpy(
                    constant::PLUGIN_CLASS_SUBCATEGORIES,
                    info.subcategories.as_mut_ptr(),
                );

                kResultOk
            }
            _ => return kInvalidArgument,
        }
    }

    unsafe fn set_host_context(&self, _context: *mut c_void) -> tresult {
        kResultOk
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
