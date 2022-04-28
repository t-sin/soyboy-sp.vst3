use std::os::raw::c_void;
use std::sync::Arc;

use vst3_com::{interfaces::IUnknown, ComInterface};
use vst3_sys::{
    utils::SharedVstPtr,
    vst::{IConnectionPoint, IHostApplication, IMessage},
    VstPtr,
};

use super::message::Vst3Message;

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
        let msg = msg.allocate(&self.host.obj());

        if let Some(msg) = msg {
            unsafe {
                let msg = std::mem::transmute::<VstPtr<dyn IMessage>, SharedVstPtr<dyn IMessage>>(
                    msg.obj(),
                );
                self.conn.notify(msg);
            }
        } else {
            println!("SoyBoyPlugin::send_message(): allocation failed");
        }
    }
}
