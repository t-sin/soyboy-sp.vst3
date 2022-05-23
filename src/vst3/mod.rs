mod controller;
mod factory;
mod gui;
mod plugin;
mod plugin_data;
mod raw_utils;
mod vst3_utils;

#[cfg(debug_assertions)]
use std::fs::File;
use std::os::raw::c_void;

#[cfg(debug_assertions)]
use simplelog::*;

pub use vst3_utils::*;

/// # Safety
///
/// This function is called by VST3 host to get VST3 plugin classes.
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn GetPluginFactory() -> *mut c_void {
    // #[cfg(debug_assertions)]
    // {
    //     #[cfg(target_os = "linux")]
    //     let path = "/home/grey/soyboy-sp.log";
    //     #[cfg(target_os = "windows")]
    //     let path = "/c/User/mostl/soyboy-sp.log";

    //     CombinedLogger::init(vec![WriteLogger::new(
    //         LevelFilter::Debug,
    //         Config::default(),
    //         File::create(path).unwrap(),
    //     )])
    //     .unwrap();
    // }

    #[cfg(debug_assertions)]
    log::debug!("GetPluginFactory(): VST3 plugin factory will be created");

    let factory = factory::SoyBoyPluginFactory::new();
    Box::into_raw(factory) as *mut c_void
}

pub fn init() {}

//// for GNU/Linux

#[cfg(target_os = "linux")]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn ModuleEntry(_: *mut c_void) -> bool {
    #[cfg(debug_assertions)]
    log::debug!("ModuleEntry(): VST3 plugin started");

    init();
    true
}

#[cfg(target_os = "linux")]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn ModuleExit() -> bool {
    #[cfg(debug_assertions)]
    log::debug!("ModuleExit(): VST3 plugin terminated");

    true
}

//// for Windows, maybe
#[cfg(target_os = "windows")]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn InitDll() -> bool {
    #[cfg(debug_assertions)]
    log::debug!("InitDll(): VST3 plugin started");

    true
}

#[cfg(target_os = "windows")]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn ExitDll() -> bool {
    #[cfg(debug_assertions)]
    log::debug!("ExitDll(): VST3 plugin terminated");

    true
}

//// for mac, maybe

#[cfg(target_os = "macos")]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn bundleEntry(_: *mut c_void) -> bool {
    true
}

#[cfg(target_os = "macos")]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn bundleExit() -> bool {
    true
}
