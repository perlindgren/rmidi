use core_foundation::runloop::CFRunLoop;

use rmidi::midi_con::*;

fn main() {
    let midi_con = ArcMutexMidiCon::new();

    println!("Sources: {:?}", midi_con.list_sources());

    midi_con.connect_source_by_index(0, move |data, mc| {
        println!("Received MIDI data from source 0: {:?}", data);
        println!("MIDI Connections state: {:?}", mc.list_sources());
    });

    midi_con.connect_source_by_index(1, move |data, mc| {
        println!("Received MIDI data from source 1: {:?}", data);
        println!("MIDI Connections state: {:?}", mc.list_sources());
    });

    midi_con.connect_destination_by_index(0);
    midi_con.send(0, &[0xc0, 0x03]); // Program Change to program 2

    midi_con.disconnect_source(1);

    CFRunLoop::run_current();

    loop {}
}
