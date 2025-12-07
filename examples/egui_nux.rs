use core_foundation::runloop::CFRunLoop;
use coremidi::{Client, Endpoint, EventList, Notification, Object, Protocol, Source, Sources};
use rmidi::midi_con::*;

// RUST_LOG="rmidi=trace" cargo run --example egui_nux

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Rust NUX - Mighty Midi Controller",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
    .unwrap();

    CFRunLoop::run_current();

    loop {}
}

struct App {
    midi_con: ArcMutexMidiCon,
    channels: [bool; 7],
    selected_channel: usize,
    show_settings: bool,
    sources: Vec<String>,
    // selected_source: usize,
}

impl App {
    fn default() -> Self {
        let midi_con = ArcMutexMidiCon::new();
        let sources = midi_con.list_sources();

        midi_con.connect_destination(0);

        Self {
            midi_con,
            channels: [true; 7],
            selected_channel: 0,
            show_settings: false,
            sources,
            // selected_source: 0,
        }
    }
}

impl App {
    fn send_program_change(&self) {
        self.midi_con.send(0, &[0xc0, self.selected_channel as u8]); // Program Change to program channel
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });

            ui.horizontal(|ui| {
                for i in 0..self.channels.len() {
                    ui.vertical(|ui| {
                        if ui
                            .radio_value(&mut self.selected_channel, i, format!("Ch {}", i + 1))
                            .clicked()
                        {
                            println!("Selected channel: {}", i + 1);
                            self.send_program_change();
                        }
                        if ui.checkbox(&mut self.channels[i], "").changed() {
                            println!("Channel {} toggled to {}", i + 1, self.channels[i]);
                        }
                    });
                }
            });

            ui.horizontal(|ui| {
                if ui.button("<<<").clicked() {
                    let mut next = self.selected_channel;
                    loop {
                        next = (next + self.channels.len() - 1) % self.channels.len();
                        if next == self.selected_channel || self.channels[next] {
                            break; // No other channels available
                        }
                    }
                    self.selected_channel = next;
                    self.send_program_change();
                    println!("Selected channel: {}", self.selected_channel);
                }

                if ui.button(">>>").clicked() {
                    let mut next = self.selected_channel;
                    loop {
                        next = (next + 1) % self.channels.len();
                        if next == self.selected_channel || self.channels[next] {
                            break; // No other channels available
                        }
                    }
                    self.selected_channel = next;
                    self.send_program_change();
                    println!("Selected channel: {}", self.selected_channel);
                }
            });

            // ui.heading("LooPer with Tap Tempo");
            // self.bars.update(ui);
            // self.meter.update(ui);
            // self.tempo.update(ui);
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

            //     if self.show_settings {
            //         egui::Window::new("Settings")
            //             .open(&mut self.show_settings)
            //             .show(ctx, |ui| {
            //                 ui.label("Select MIDI Source:");

            //                 // Example: get source names (replace with your actual source list)
            //                 let midi_sources: Sources = Sources;
            //                 let sources: Vec<String> = midi_sources
            //                     .into_iter()
            //                     .map(|s| s.display_name().unwrap_or_else(|| "Unknown".to_string()))
            //                     .collect();

            //                 // println!("{}", sources.join(", "));

            //                 egui::ComboBox::from_label("MIDI Source")
            //                     .selected_text(
            //                         sources
            //                             .get(self.selected_source)
            //                             .cloned()
            //                             .unwrap_or_else(|| "None".to_string()),
            //                     )
            //                     .show_ui(ui, |ui| {
            //                         for (i, name) in sources.iter().enumerate() {
            //                             ui.selectable_value(&mut self.selected_source, i, name);
            //                         }
            //                     });

            //                 // You can use self.selected_source to update your Connections, etc.
            //             });
            //    }
            // })
        });
    }
}
