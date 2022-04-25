use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr::null_mut;

use vst3_com::{interfaces::IUnknown, ComInterface};
use vst3_sys::{
    base::{kInvalidArgument, kNotImplemented, kResultOk},
    utils::SharedVstPtr,
    vst::{IHostApplication, IMessage},
    VstPtr,
};

use crate::vst3::utils::ComPtr;

pub enum Vst3Message {
    NoteOn,
}

impl Vst3Message {
    fn to_string(&self) -> String {
        match self {
            Vst3Message::NoteOn => "vst3:note-on".to_string(),
        }
    }

    fn to_cstring(&self) -> CString {
        CString::new(self.to_string()).unwrap()
    }

    fn write_message(&self, msg: &mut VstPtr<dyn IMessage>) {
        match self {
            Vst3Message::NoteOn => {
                unsafe { msg.set_message_id(self.to_cstring().as_ptr()) };
            }
        }
    }

    pub fn allocate(&self, host: &VstPtr<dyn IHostApplication>) -> Option<ComPtr<dyn IMessage>> {
        #[cfg(debug_assertions)]
        println!("Vst3Message::allocate()");

        let iid = <dyn IMessage as ComInterface>::IID;
        let mut msg_ptr: *mut c_void = null_mut();

        #[cfg(debug_assertions)]
        {
            let mut name: [u16; 1024] = [0; 1024];
            let result = unsafe { host.get_name(&mut name as *mut u16) };
            println!(
                "Vst3Message::allocate(): Debug: host application name = {}",
                String::from_utf16(&name).unwrap()
            );
        }

        #[cfg(debug_assertions)]
        println!("Vst3Message::allocate(): calling IHostApplication::create_instance()");
        let result = unsafe { host.create_instance(iid, iid, &mut msg_ptr as *mut _) };
        if result != kResultOk {
            #[cfg(debug_assertions)]
            print!("Vst3Message::allocate(): calling IHostApplication::create_instance() failed because ");

            return None;
        }

        #[cfg(debug_assertions)]
        println!("Vst3Message::allocate(): calling IHostApplication::create_instance() succeeded");

        let mut msg_obj = unsafe { VstPtr::shared(msg_ptr as *mut _).unwrap() };
        #[cfg(debug_assertions)]
        self.write_message(&mut msg_obj);
        println!("Vst3Message::allocate(): message is written into IMessage");

        Some(ComPtr::new(msg_ptr, msg_obj))
    }
}
