mod controller;
mod factory;
mod gui;
mod message;
mod plugin;
mod plugin_data;
mod utils;

use std::os::raw::c_void;

/// # Safety
///
/// This function is called by VST3 host to get VST3 plugin classes.
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn GetPluginFactory() -> *mut c_void {
    #[cfg(debug_assertions)]
    println!("GetPluginFactory(): VST3 plugin factory will be created");

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
    println!("ModuleEntry(): VST3 plugin started");

    init();
    true
}

#[cfg(target_os = "linux")]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn ModuleExit() -> bool {
    #[cfg(debug_assertions)]
    println!("ModuleExit(): VST3 plugin terminated");

    true
}

//// for Windows, maybe
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn InitDll() -> bool {
    #[cfg(debug_assertions)]
    println!("InitDll(): VST3 plugin started");

    true
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn ExitDll() -> bool {
    #[cfg(debug_assertions)]
    println!("ExitDll(): VST3 plugin terminated");

    true
}

//// for mac, maybe

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn bundleEntry(_: *mut c_void) -> bool {
    true
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn bundleExit() -> bool {
    true
}
