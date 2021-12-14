mod controller;
mod factory;
mod gui;
mod plugin;
mod plugin_data;
mod utils;

use std::os::raw::c_void;

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn GetPluginFactory() -> *mut c_void {
    Box::into_raw(factory::SoyBoyPluginFactory::new()) as *mut c_void
}

pub fn init() {}

//// for GNU/Linux

#[cfg(target_os = "linux")]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn ModuleEntry(_: *mut c_void) -> bool {
    init();
    true
}

#[cfg(target_os = "linux")]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn ModuleExit() -> bool {
    true
}

//// for Windows, maybe
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn InitDll() -> bool {
    true
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn ExitDll() -> bool {
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
