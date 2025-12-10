use std::mem::MaybeUninit;

use windows::Win32::Media::Audio::{HMIDIIN, MIDI_WAVE_OPEN_TYPE};
use windows::Win32::Media::Audio::{midiInGetNumDevs, midiInOpen};
fn main() {
    let mut h: MaybeUninit<HMIDIIN> = MaybeUninit::uninit();
    println!("Connected MIDI In Devices: {}", unsafe {
        midiInGetNumDevs()
    });
    unsafe {
        midiInOpen(
            h.as_mut_ptr(),
            0,
            Option::Some(&mut callback as *mut _ as usize),
            None,
            MIDI_WAVE_OPEN_TYPE(0),
        );
    }
}

extern "C" fn callback(
    _h_midi_in: HMIDIIN,
    _w_msg: usize,
    _dw_instance: usize,
    _dw_param1: usize,
    _dw_param2: usize,
) {
}
