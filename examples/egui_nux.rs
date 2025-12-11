use core_foundation::runloop::CFRunLoop;
use rmidi::midi_con::*;
use std::{
    collections::HashMap,
    //hash::Hash,
    sync::{Arc, Mutex},
};

// RUST_LOG="rmidi=trace" cargo run --example egui_nux

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 640.0]),
        ..Default::default()
    };

    let app = ArcMutexApp::new();
    let cb_app = app.clone();
    let midi_con = cb_app.0.lock().unwrap().midi_con.clone();
    midi_con.set_notification_callback(move |notification: &Notification| {
        cb_app.notification_callback(notification);
    });

    eframe::run_native(
        "Rust NUX - Mighty Midi Controller",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
    .unwrap();

    CFRunLoop::run_current();

    loop {}
}

enum Learn {
    None,
    PreviousChannel,
    NextChannel,
}

struct MidiMapper {
    previous: Option<Vec<u8>>,
    next: Option<Vec<u8>>,
}
struct App {
    midi_con: ArcMutexMidiCon,
    channels: [bool; 7],
    selected_channel: usize,
    show_settings: bool,
    sources: Vec<(usize, bool, String)>,
    destinations: Vec<(usize, bool, String)>,
    selected_source: usize,
    learn: Learn,
    midi_map: HashMap<usize, MidiMapper>,
}

#[derive(Clone)]
struct ArcMutexApp(Arc<Mutex<App>>);

impl ArcMutexApp {
    fn notification_callback(&self, notification: &Notification) {
        println!("App received notification: {:?}", notification);
    }

    fn new() -> Self {
        let midi_con = ArcMutexMidiCon::new();
        midi_con.connect_destination_by_index(0);

        let sources = midi_con.list_sources();
        let destinations = midi_con.list_destinations();

        ArcMutexApp(Arc::new(Mutex::new(App {
            midi_con,
            channels: [true; 7],
            selected_channel: 0,
            show_settings: false,
            sources,
            destinations,
            selected_source: 0,
            learn: Learn::None,
            midi_map: HashMap::new(),
        })))
    }
}

impl App {
    fn send_program_change(&self) {
        self.midi_con.send(0, &[0xc0, self.selected_channel as u8]); // Program Change to program channel
    }
}

impl eframe::App for ArcMutexApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let app = &mut *self.0.lock().unwrap();
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });

            ui.heading("Mighty Midi Controller");

            // if ui.button("⚙️").clicked() {
            //     println!(" clicked ");
            //     self.show_settings = true;
            // }

            // if self.show_settings {
            ui.label("Select MIDI Source:");
            egui::Grid::new("channel_table")
                .striped(true)
                .show(ui, |ui| {
                    ui.heading("In Connected");
                    ui.heading("Out Connected");
                    ui.heading("Name");
                    ui.end_row();

                    for (i, connected, name) in app.sources.iter_mut() {
                        if ui.checkbox(&mut *connected, "").changed() {
                            if *connected {
                                let source_index = *i;
                                let cb_app = self.clone();
                                app.midi_con.connect_source_by_index(
                                    source_index,
                                    move |data, midi_con| {
                                        let app = &mut *cb_app.0.lock().unwrap();
                                        println!(
                                            "Received MIDI data from source {}: {:?}",
                                            source_index, data
                                        );
                                        midi_con.send(0, data);
                                        match app.learn {
                                            Learn::None => {
                                                // Normal MIDI handling
                                                if let Some(map) = app.midi_map.get(&source_index) {
                                                    if let Some(prev) = &map.previous {
                                                        if data == prev.as_slice() {
                                                            // Previous channel
                                                            let mut next = app.selected_channel;
                                                            loop {
                                                                next = (next + app.channels.len()
                                                                    - 1)
                                                                    % app.channels.len();
                                                                if next == app.selected_channel
                                                                    || app.channels[next]
                                                                {
                                                                    break; // No other channels available
                                                                }
                                                            }
                                                            app.selected_channel = next;
                                                            app.send_program_change();
                                                            println!(
                                                                "Selected channel: {}",
                                                                app.selected_channel
                                                            );
                                                        }
                                                    }
                                                    if let Some(next) = &map.next {
                                                        if data == next.as_slice() {
                                                            // Next channel
                                                            let mut next = app.selected_channel;
                                                            loop {
                                                                next =
                                                                    (next + 1) % app.channels.len();
                                                                if next == app.selected_channel
                                                                    || app.channels[next]
                                                                {
                                                                    break; // No other channels available
                                                                }
                                                            }
                                                            app.selected_channel = next;
                                                            app.send_program_change();
                                                            println!(
                                                                "Selected channel: {}",
                                                                app.selected_channel
                                                            );
                                                            return;
                                                        }
                                                    }
                                                }
                                            }
                                            _ => {
                                                // MIDI Learn logic
                                                let v = data.to_vec();
                                                let map = app
                                                    .midi_map
                                                    .entry(source_index)
                                                    .or_insert(MidiMapper {
                                                        previous: None,
                                                        next: None,
                                                    });

                                                match app.learn {
                                                    Learn::PreviousChannel => {
                                                        println!("Learn Previous Channel");
                                                        map.previous = Some(v);
                                                    }
                                                    Learn::NextChannel => {
                                                        println!("Learn Next Channel");
                                                        map.next = Some(v);
                                                    }
                                                    _ => unreachable!(),
                                                }
                                                app.learn = Learn::None;
                                            }
                                        }
                                    },
                                );
                            } else {
                                app.midi_con.disconnect_source(*i);
                            }
                        }
                        ui.label(format!("{}", name));

                        ui.end_row();
                    }
                });

            ui.separator();

            ui.horizontal(|ui| {
                for i in 0..app.channels.len() {
                    ui.vertical(|ui| {
                        if ui
                            .radio_value(&mut app.selected_channel, i, format!("Ch {}", i + 1))
                            .clicked()
                        {
                            println!("Selected channel: {}", i + 1);
                            app.send_program_change();
                        }
                        if ui.checkbox(&mut app.channels[i], "").changed() {
                            println!("Channel {} toggled to {}", i + 1, app.channels[i]);
                        }
                    });
                }
            });

            ui.horizontal(|ui| {
                if ui.button("<<<").clicked() {
                    if ui.input(|i| i.modifiers.shift) {
                        // midi learn
                        println!("Learn <<<");
                        app.learn = Learn::PreviousChannel;
                    } else {
                        let mut next = app.selected_channel;
                        loop {
                            next = (next + app.channels.len() - 1) % app.channels.len();
                            if next == app.selected_channel || app.channels[next] {
                                break; // No other channels available
                            }
                        }
                        app.selected_channel = next;
                        app.send_program_change();
                        println!("Selected channel: {}", app.selected_channel);
                    }
                }

                if ui.button(">>>").clicked() {
                    if ui.input(|i| i.modifiers.shift) {
                        // midi learn
                        println!("Learn >>>");
                        app.learn = Learn::NextChannel;
                    } else {
                        let mut next = app.selected_channel;
                        loop {
                            next = (next + 1) % app.channels.len();
                            if next == app.selected_channel || app.channels[next] {
                                break; // No other channels available
                            }
                        }
                        app.selected_channel = next;
                        app.send_program_change();
                        println!("Selected channel: {}", app.selected_channel);
                    }
                }
            });

            // ui.separator();
            // ui.horizontal_top(|ui| {
            //     ui.label(format!("{:?}", self.connections.source));
            //     //         ui.horizontal_top(|ui| {
            //     if ui.button("⚙️").clicked() {
            //         println!(" clicked ");
            //         self.show_settings = true;
            //     }
            //     //     ui.label(format!("{:?}", self.connections.source));
            //     // });

            //
        });
    }
}
