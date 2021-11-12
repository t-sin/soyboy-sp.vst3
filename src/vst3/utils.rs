use std::{
    os::raw::{c_char, c_short, c_void},
    ptr::copy_nonoverlapping,
};
use widestring::U16CString;

use vst3_sys::vst::{
    BusDirections, BusFlags, BusInfo, BusTypes, DataEvent, Event, EventData, EventTypes,
    MediaTypes, ParameterInfo, String128, TChar,
};

pub unsafe fn strcpy(src: &str, dst: *mut c_char) {
    copy_nonoverlapping(src.as_ptr() as *const c_void as *const _, dst, src.len());
}

pub unsafe fn wstrcpy(src: &str, dst: *mut c_short) {
    let src = U16CString::from_str(src).unwrap();
    let mut src = src.into_vec();
    src.push(0);
    copy_nonoverlapping(src.as_ptr() as *const c_void as *const _, dst, src.len());
}

pub fn str128cpy(src: &String128, dest: &mut String128) {
    for idx in 0..src.len() {
        dest[idx] = src[idx];
    }
}

pub unsafe fn tcharcpy(src: &str, dst: *mut TChar) {
    let mut ptr = dst;
    for c in src.chars() {
        *ptr = c as i16;
        ptr = ptr.add(1);
    }
    *ptr = 0;
}

pub fn tchar_to_string(src: *const TChar) -> String {
    let mut ptr = src;
    let mut chars = Vec::new();

    unsafe {
        while *ptr != 0 {
            chars.push(*ptr as u16);
            ptr = ptr.add(1);
        }
    }

    if let Ok(s) = String::from_utf16(&chars) {
        s
    } else {
        "".to_string()
    }
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

const K_NOTE_ON_EVENT: u16 = EventTypes::kNoteOnEvent as u16;
const K_NOTE_OFF_EVENT: u16 = EventTypes::kNoteOffEvent as u16;

pub fn as_event_type(n: u16) -> Option<EventTypes> {
    match n {
        K_NOTE_ON_EVENT => Some(EventTypes::kNoteOnEvent),
        K_NOTE_OFF_EVENT => Some(EventTypes::kNoteOffEvent),
        _ => None,
    }
}

pub fn make_empty_event() -> Event {
    let bytes = [0];
    Event {
        bus_index: 0,
        sample_offset: 0,
        ppq_position: 0.0,
        flags: 0,
        type_: 0,
        event: EventData {
            data: DataEvent {
                size: 0,
                type_: 0,
                bytes: bytes.as_ptr(),
            },
        },
    }
}

pub fn make_empty_bus_info() -> BusInfo {
    BusInfo {
        name: [0; 128],
        media_type: MediaTypes::kAudio as i32,
        direction: BusDirections::kInput as i32,
        channel_count: 0,
        bus_type: BusTypes::kMain as i32,
        flags: BusFlags::kDefaultActive as u32,
    }
}

pub fn make_empty_param_info() -> ParameterInfo {
    ParameterInfo {
        id: 0,
        title: [0; 128],
        short_title: [0; 128],
        units: [0; 128],
        step_count: 1,
        default_normalized_value: 0.0,
        unit_id: 0,
        flags: 0,
    }
}
