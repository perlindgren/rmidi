use windows::Win32::Media::Audio::midiInGetNumDevs;
fn main() {
    println!("Connected MIDI In Devices: {}", unsafe {
        midiInGetNumDevs()
    });
}
