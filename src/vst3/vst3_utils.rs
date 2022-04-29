use std::os::raw::c_void;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};

use vst3_com::{interfaces::IUnknown, ComInterface};
use vst3_sys::{
    base::kResultOk,
    utils::SharedVstPtr,
    vst::{IConnectionPoint, IHostApplication, IMessage},
    VstPtr,
};

use super::message::Vst3Message;

pub struct SyncPtr<I: ComInterface + ?Sized> {
    ptr: VstPtr<I>,
}

unsafe impl<I: ComInterface + ?Sized> Sync for SyncPtr<I> {}
unsafe impl<I: ComInterface + ?Sized> Send for SyncPtr<I> {}

impl<I: ComInterface + ?Sized> SyncPtr<I> {
    pub fn new(ptr: VstPtr<I>) -> Self {
        Self { ptr }
    }

    pub fn ptr(&self) -> &VstPtr<I> {
        &self.ptr
    }
}

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

pub fn get_host_app(context: &VstPtr<dyn IUnknown>) -> ComPtr<dyn IHostApplication> {
    let host_iid = <dyn IHostApplication as ComInterface>::IID;
    let mut host_ptr: *mut c_void = null_mut();

    let result = unsafe { context.query_interface(&host_iid as *const _, &mut host_ptr as *mut _) };

    if result != kResultOk {
        panic!("host context is not implemented IHostApplication");
    }

    let host_obj = unsafe { VstPtr::shared(host_ptr as *mut _).unwrap() };

    ComPtr::new(host_ptr, host_obj)
}

pub fn send_message(
    host_context: Arc<Mutex<SyncPtr<dyn IUnknown>>>,
    controller: Arc<Mutex<SyncPtr<dyn IConnectionPoint>>>,
    msg: Vst3Message,
) {
    let controller = controller.lock().unwrap();
    let controller = controller.ptr();
    let context = host_context.lock().unwrap();
    let context = context.ptr();
    let host = get_host_app(&context).obj();

    let msg = msg.allocate(&host);
    if let Some(msg) = msg {
        unsafe {
            let msg =
                std::mem::transmute::<VstPtr<dyn IMessage>, SharedVstPtr<dyn IMessage>>(msg.obj());
            controller.notify(msg);
        }
    } else {
        println!("SoyBoyPlugin::send_message(): allocation failed");
    }
}
