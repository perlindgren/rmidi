use core_foundation::runloop::CFRunLoop;

use rmidi::midi_con::*;

struct App {}

impl App {
    fn new() -> Self {
        App {}
    }

    fn notification_callback(&self, notification: &Notification) {
        println!("App received notification: {:?}", notification);
    }
}

fn main() {
    let app = App::new();
    let midi_con = ArcMutexMidiCon::new();
    midi_con.set_notification_callback(move |notification: &Notification| {
        app.notification_callback(notification);
    });

    println!("Sources: {:?}", midi_con.list_sources());

    midi_con.connect_source_by_index(0, move |data, mc| {
        println!("Received MIDI data from source 0: {:?}", data);
        println!("MIDI Connections state: {:?}", mc.list_sources());
    });

    midi_con.connect_destination_by_index(0);
    midi_con.send(0, &[0xc0, 0x03]); // Program Change to program 2

    CFRunLoop::run_current();
    loop {}
}
