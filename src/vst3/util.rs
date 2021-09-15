use std::{
    os::raw::{c_char, c_short, c_void},
    ptr::copy_nonoverlapping,
};
use widestring::U16CString;

use vst3_sys::vst::{BusDirections, MediaTypes};

pub unsafe fn strcpy(src: &str, dst: *mut c_char) {
    copy_nonoverlapping(src.as_ptr() as *const c_void as *const _, dst, src.len());
}

pub unsafe fn wstrcpy(src: &str, dst: *mut c_short) {
    let src = U16CString::from_str(src).unwrap();
    let mut src = src.into_vec();
    src.push(0);
    copy_nonoverlapping(src.as_ptr() as *const c_void as *const _, dst, src.len());
}

const K_AUDIO: i32 = MediaTypes::kAudio as i32;
const K_EVENT: i32 = MediaTypes::kEvent as i32;
const K_NUM_MEDIA_TYPES: i32 = MediaTypes::kNumMediaTypes as i32;

pub fn as_media_type(n: i32) -> Option<MediaTypes> {
    match n {
        K_AUDIO => Some(MediaTypes::kAudio),
        K_EVENT => Some(MediaTypes::kEvent),
        K_NUM_MEDIA_TYPES => Some(MediaTypes::kNumMediaTypes),
        _ => None,
    }
}

const K_INPUT: i32 = BusDirections::kInput as i32;
const K_OUTPUT: i32 = BusDirections::kOutput as i32;

pub fn as_bus_dir(n: i32) -> Option<BusDirections> {
    match n {
        K_INPUT => Some(BusDirections::kInput),
        K_OUTPUT => Some(BusDirections::kOutput),
        _ => None,
    }
}
