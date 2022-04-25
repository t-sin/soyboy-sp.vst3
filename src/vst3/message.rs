use std::ffi::CString;
use std::fmt;
use std::os::raw::c_void;
use std::ptr::null_mut;

use vst3_com::ComInterface;
use vst3_sys::{
    base::kResultOk,
    vst::{IHostApplication, IMessage},
    VstPtr,
};

use crate::vst3::utils::ComPtr;

pub enum Vst3Message {
    NoteOn,
}

impl fmt::Display for Vst3Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Vst3Message::NoteOn => "vst3:note-on",
        };

        write!(f, "{}", s)
    }
}

impl Vst3Message {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "vst3:note-on" => Some(Vst3Message::NoteOn),
            _ => None,
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
        let iid = &iid as *const _;
        let mut msg_ptr: *mut c_void = null_mut();

        let result = unsafe { host.create_instance(iid, iid, &mut msg_ptr as *mut _) };
        if result != kResultOk {
            #[cfg(debug_assertions)]
            print!("Vst3Message::allocate(): calling IHostApplication::create_instance() failed because ");

            return None;
        }

        let mut msg_obj = unsafe { VstPtr::shared(msg_ptr as *mut _).unwrap() };
        #[cfg(debug_assertions)]
        self.write_message(&mut msg_obj);

        Some(ComPtr::new(msg_ptr, msg_obj))
    }
}
