use std::os::raw::c_void;

use vst3_com::sys::GUID;
use vst3_sys::{
    base::{
        kInvalidArgument, kResultFalse, kResultOk, tresult, FactoryFlags, IPluginFactory,
        IPluginFactory2, IPluginFactory3, PClassInfo, PClassInfo2, PClassInfoW, PFactoryInfo,
    },
    VST3,
};

use crate::gbi::GameBoyInstrument;
use crate::vst3::{
    controller::GameBoyController,
    plugin::GameBoyPlugin,
    plugin_data,
    util::{strcpy, wstrcpy},
};

#[VST3(implements(IPluginFactory3))]
pub struct GameBoyPluginFactory {}

impl GameBoyPluginFactory {
    pub fn new() -> Box<Self> {
        Self::allocate()
    }
}

impl IPluginFactory for GameBoyPluginFactory {
    unsafe fn get_factory_info(&self, info: *mut PFactoryInfo) -> tresult {
        let info = &mut *info;

        strcpy(plugin_data::VST3_VENDOR, info.vendor.as_mut_ptr());
        strcpy(plugin_data::VST3_URL, info.url.as_mut_ptr());
        strcpy(plugin_data::VST3_EMAIL, info.email.as_mut_ptr());

        info.flags = FactoryFlags::kComponentNonDiscardable as i32;

        kResultOk
    }

    unsafe fn count_classes(&self) -> i32 {
        2
    }

    unsafe fn get_class_info(&self, idx: i32, info: *mut PClassInfo) -> tresult {
        match idx {
            0 => {
                let info = &mut *info;

                info.cardinality = 0x7FFF_FFFF;
                info.cid = GameBoyPlugin::CID;

                strcpy(plugin_data::VST3_CLASS_NAME, info.name.as_mut_ptr());
                strcpy(plugin_data::VST3_CLASS_CATEGORY, info.category.as_mut_ptr());
                strcpy(plugin_data::VST3_CLASS_NAME, info.name.as_mut_ptr());

                kResultOk
            }
            1 => {
                let info = &mut *info;

                info.cardinality = 0x7FFF_FFFF;
                info.cid = GameBoyController::CID;

                strcpy(
                    plugin_data::VST3_CONTROLLER_CLASS_NAME,
                    info.name.as_mut_ptr(),
                );
                strcpy(
                    plugin_data::VST3_CONTROLLER_CLASS_CATEGORY,
                    info.category.as_mut_ptr(),
                );
                strcpy(
                    plugin_data::VST3_CONTROLLER_CLASS_NAME,
                    info.name.as_mut_ptr(),
                );

                kResultOk
            }
            _ => kInvalidArgument,
        }
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
                let ptr =
                    Box::into_raw(GameBoyPlugin::new(GameBoyInstrument::new())) as *mut c_void;
                *obj = ptr;

                kResultOk
            }
            GameBoyController::CID => {
                let ptr = Box::into_raw(GameBoyController::new()) as *mut c_void;
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

                info.class_flags = 1;
                strcpy(plugin_data::VST3_CLASS_NAME, info.name.as_mut_ptr());
                strcpy(plugin_data::VST3_VENDOR, info.vendor.as_mut_ptr());
                strcpy(plugin_data::VST3_VERSION, info.version.as_mut_ptr());
                strcpy(plugin_data::VST3_CLASS_CATEGORY, info.category.as_mut_ptr());
                strcpy(
                    plugin_data::VST3_CLASS_SUBCATEGORIES,
                    info.subcategories.as_mut_ptr(),
                );

                kResultOk
            }
            1 => {
                let info = &mut *info;

                info.class_flags = 0;
                strcpy(
                    plugin_data::VST3_CONTROLLER_CLASS_NAME,
                    info.name.as_mut_ptr(),
                );
                strcpy(plugin_data::VST3_VENDOR, info.vendor.as_mut_ptr());
                strcpy(plugin_data::VST3_VERSION, info.version.as_mut_ptr());
                strcpy(
                    plugin_data::VST3_CONTROLLER_CLASS_CATEGORY,
                    info.category.as_mut_ptr(),
                );
                strcpy(
                    plugin_data::VST3_CONTROLLER_CLASS_SUBCATEGORIES,
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

                info.class_flags = 1;
                wstrcpy(plugin_data::VST3_CLASS_NAME, info.name.as_mut_ptr());
                wstrcpy(plugin_data::VST3_VENDOR, info.vendor.as_mut_ptr());
                wstrcpy(plugin_data::VST3_VERSION, info.version.as_mut_ptr());
                strcpy(plugin_data::VST3_CLASS_CATEGORY, info.category.as_mut_ptr());
                strcpy(
                    plugin_data::VST3_CLASS_SUBCATEGORIES,
                    info.subcategories.as_mut_ptr(),
                );

                kResultOk
            }
            1 => {
                let info = &mut *info;

                info.class_flags = 0;
                wstrcpy(
                    plugin_data::VST3_CONTROLLER_CLASS_NAME,
                    info.name.as_mut_ptr(),
                );
                wstrcpy(plugin_data::VST3_VENDOR, info.vendor.as_mut_ptr());
                wstrcpy(plugin_data::VST3_VERSION, info.version.as_mut_ptr());
                strcpy(
                    plugin_data::VST3_CONTROLLER_CLASS_CATEGORY,
                    info.category.as_mut_ptr(),
                );
                strcpy(
                    plugin_data::VST3_CONTROLLER_CLASS_SUBCATEGORIES,
                    info.subcategories.as_mut_ptr(),
                );

                kResultOk
            }
            _ => kInvalidArgument,
        }
    }

    unsafe fn set_host_context(&self, _context: *mut c_void) -> tresult {
        kResultOk
    }
}
