use std::ffi::CString;
use std::ptr::null_mut;

use vst3_com::{ComInterface, RawVstPtr, VstPtr};

use vst3_sys::{
    base::{kResultOk, tchar, tresult, FIDString},
    utils::SharedVstPtr,
    vst::{AttrID, IAttributeList, IHostApplication, IMessage},
    VST3,
};

use crate::vst3::utils;

pub enum Vst3Message {
    NoteOn,
}

impl Vst3Message {
    fn to_cstring(&self) -> CString {
        match self {
            Vst3Message::NoteOn => CString::new("vst3:note-on").unwrap(),
        }
    }

    pub fn set(&self, msg: &mut SharedVstPtr<dyn IMessage>) {
        let msg_id = self.to_cstring();
        unsafe {
            msg.upgrade().unwrap().set_message_id(msg_id.as_ptr());
        }
        println!("set_message_id() OK");

        match self {
            Vst3Message::NoteOn => (),
        }
    }

    pub fn allocate(
        &self,
        host: &RawVstPtr<dyn IHostApplication>,
    ) -> Option<SharedVstPtr<dyn IMessage>> {
        // let iid = <dyn IMessage>::IID;
        // let mut msg: *mut *mut <I as ComInterface>::VTable = null_mut();

        // let result = unsafe { host.create_instance(iid, iid, msg as *mut *mut _) };
        // println!("create_instance()");
        // if result != kResultOk {
        //     println!("create_instance() NG");
        //     return None;
        // }
        // println!("create_instance() OK");

        // let msg_id = self.to_cstring();
        // let msg = SharedVstPtr::<dyn IMessage>::new(msg);
        // unsafe {
        //     msg.upgrade().unwrap().set_message_id(msg_id.as_ptr());
        // }
        // println!("set_message_id() OK");

        // match self {
        //     Vst3Message::NoteOn => (),
        // }

        // Some(msg)
        None
    }
}
