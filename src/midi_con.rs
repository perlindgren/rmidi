use coremidi::{
    Client, Destination, Destinations, InputPort, Notification, OutputPort, PacketBuffer,
    PacketList, Source, Sources,
};
use log::trace;

use std::collections::HashMap;
use std::marker::Send;
use std::sync::{Arc, Mutex};

pub struct MidiCon {
    pub opt_client: Option<Client>,
    pub in_ports: HashMap<usize, InputPort>,
    pub out_ports: HashMap<usize, OutputPort>,
}

#[derive(Clone)]
pub struct ArcMutexMidiCon(pub Arc<Mutex<MidiCon>>);

impl ArcMutexMidiCon {
    /// Create MidiConnections without a client. Client will be set later.
    pub fn new() -> Self {
        let arc_mutex_midi_con = ArcMutexMidiCon(Arc::new(Mutex::new(MidiCon {
            opt_client: None,
            in_ports: HashMap::new(),
            out_ports: HashMap::new(),
        })));
        let mut cb = arc_mutex_midi_con.clone();

        let client =
            Client::new_with_notifications("Example Client", move |notification: &Notification| {
                cb.update_connections(notification);
            })
            .unwrap();

        arc_mutex_midi_con.0.lock().unwrap().opt_client = Some(client);
        arc_mutex_midi_con
    }

    pub fn update_connections(&mut self, notification: &Notification) {
        println!("notification: {:?}", notification);
    }

    /// Connect to a MIDI source by its index with a callback for incoming data
    pub fn connect_source(
        &self,
        source_index: usize,
        cb: impl Fn(&PacketList, &mut MidiCon) -> () + Send + 'static,
    ) {
        let midi_con = &mut self.0.lock().unwrap();
        if let Some(client) = &midi_con.opt_client {
            trace!("Connecting to source index: {}", source_index);
            let source = Source::from_index(source_index).unwrap();
            let mc = self.clone();
            let input_port = client
                .input_port("input", move |packet_list| {
                    let mut mc = mc.0.lock().unwrap();
                    cb(packet_list, &mut *mc);
                })
                .unwrap();
            input_port.connect_source(&source).unwrap();
            println!("Connected to source: {}", source.display_name().unwrap());
            midi_con.in_ports.insert(source_index, input_port);
        }
    }

    /// List available MIDI sources by their display names
    pub fn list_sources(&self) -> Vec<String> {
        trace!("Listing MIDI Sources:");
        Sources
            .into_iter()
            .map(|source| {
                source
                    .display_name()
                    .unwrap_or_else(|| "Unknown".to_string())
            })
            .collect()
    }

    /// List available MIDI destinations by their display names
    pub fn list_destinations(&self) -> Vec<String> {
        trace!("Listing MIDI Destinations:");
        Destinations
            .into_iter()
            .map(|source| {
                source
                    .display_name()
                    .unwrap_or_else(|| "Unknown".to_string())
            })
            .collect()
    }

    /// Connect to a MIDI destination by its index
    pub fn connect_destination(&self, destination_index: usize) {
        let midi_con = &mut self.0.lock().unwrap();
        if let Some(client) = &midi_con.opt_client {
            trace!("Connecting to destination index: {}", destination_index);
            let destination = Destination::from_index(destination_index).unwrap();
            let output_port = client.output_port("output").unwrap();
            midi_con.out_ports.insert(destination_index, output_port);
            trace!(
                "Connected to destination: {}",
                destination.display_name().unwrap()
            );
        }
    }

    /// Send MIDI data to a connected destination by its index
    pub fn send(&self, destination_index: usize, data: &[u8]) {
        let midi_con = &mut self.0.lock().unwrap();
        if let Some(output_port) = midi_con.out_ports.get(&destination_index) {
            let destination = Destination::from_index(destination_index).unwrap();
            output_port
                .send(&destination, &PacketBuffer::new(0, data))
                .unwrap();

            trace!(
                "Sent MIDI data to destination index {}: {:?}",
                destination_index, data
            );
        }
    }

    pub fn disconnect_source(&self, source_index: usize) {
        let midi_con = &mut self.0.lock().unwrap();
        if let Some(input_port) = midi_con.in_ports.remove(&source_index) {
            drop(input_port);
            trace!("Disconnected from source index: {}", source_index);
        }
    }
}
