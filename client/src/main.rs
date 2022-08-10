#![cfg_attr(
    all(target_os = "windows", not(feature = "console"),),
    windows_subsystem = "windows"
)]
use std::{sync::Arc, time::Duration};

use eframe::IconData;
use egui::{Button, Color32, FontFamily, FontId, RichText, TextEdit, TextStyle};

use fast_mic::{
    common::{UserAction, Communicator, GuiStatus, LoopMessage},
    event_loop::start_event_loop,
};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref RED: Color32 = Color32::from_rgb(149, 1, 1);
    static ref GREEN: Color32 = Color32::from_rgb(16, 148, 54);
    static ref YELLOW: Color32 = Color32::from_rgb(148, 120, 16);
    static ref ICON_BYTES: &'static [u8] = include_bytes!("assets/icon.png");
}

fn main() {
    let size = Some(egui::vec2(400.0, 250.0));
    let img = image::load_from_memory(&ICON_BYTES)
        .expect("Fail loading icon")
        .into_bytes();

    let icon_data = IconData {
        rgba: img,
        width: 128,
        height: 128,
    };

    let options = eframe::NativeOptions {
        min_window_size: size,
        max_window_size: size,
        initial_window_size: size,
        icon_data: Some(icon_data),
        // resizable: false,
        ..Default::default()
    };
    eframe::run_native("Fast Mic", options, Box::new(|cc| Box::new(MyApp::new(cc))));
}

pub struct MyApp {
    address: String,
    status: GuiStatus,
    comm: Communicator<UserAction, LoopMessage>,
    error_message: Option<String>,
}

impl eframe::App for MyApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("address", self.address.to_owned());
        storage.flush();
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        Duration::from_secs(120)
    }

    fn on_exit(&mut self, _gl: &eframe::glow::Context) {
        if let Err(err) = self.comm.send(UserAction::Exit) {
            println!("Error sending message: {}", err);
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.status.can_connect() {
            if let Ok(message) = self.comm.try_receive() {
                match message {
                    LoopMessage::Ready => self.status = GuiStatus::Ready,
                    LoopMessage::SocketConnected => self.status = GuiStatus::Connected,
                    LoopMessage::SocketCannotConnect => {
                        self.status = GuiStatus::Ready;
                        self.error_message = Some("Error connecting".to_string());
                    }
                    LoopMessage::SocketClosed => {
                        self.status = GuiStatus::Ready;
                    }
                    LoopMessage::SocketReconnecting => {
                        self.status = GuiStatus::Reconnecting;
                    }
                    LoopMessage::AudioStreamError(error) => {
                        self.status = GuiStatus::Failed;
                        self.error_message = Some(error)
                    }
                }
            }
        }

        let address = self.address.to_owned();
        let text_edit = TextEdit::singleline(&mut self.address)
            .desired_width(160.0)
            .interactive(self.status.can_connect());
        let button = Button::new(get_button_text(&self.status)).sense(
            if self.status.can_connect() || self.status == GuiStatus::Connected {
                egui::Sense::click()
            } else {
                egui::Sense::focusable_noninteractive()
            },
        );

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                ui.heading(
                    RichText::new(get_status_text(&self.status))
                        .color(get_text_color(&self.status)),
                );
                ui.add_space(10.0);
                ui.add(text_edit);
                ui.add_space(10.0);
                if ui.add(button).clicked() {
                    self.error_message = None;
                    if self.status == GuiStatus::Connected {
                        match self.comm.send(UserAction::UserDisconnect) {
                            Ok(_) => {
                                self.status = GuiStatus::Disconnecting;
                            }
                            Err(err) => {
                                eprintln!("Communicator error: {}", err);
                                self.status = GuiStatus::Failed;
                                self.error_message = Some("Communicator error".to_string());
                            }
                        }
                    } else {
                        match self.comm.send(UserAction::Connect(address)) {
                            Ok(_) => {
                                self.status = GuiStatus::Connecting;
                            }
                            Err(err) => {
                                eprintln!("Communicator error: {}", err);
                                self.status = GuiStatus::Failed;
                                self.error_message = Some("Communicator error".to_string());
                            }
                        }
                    }
                };
                if let Some(error_message) = self.error_message.as_ref() {
                    ui.add_space(20.0);
                    ui.label(error_message);
                }
            });
        });
    }
}

fn get_text_color(status: &GuiStatus) -> Color32 {
    match status {
        GuiStatus::Disconnecting
        | GuiStatus::Connecting
        | GuiStatus::Reconnecting => *YELLOW,
        GuiStatus::Ready | GuiStatus::Connected => *GREEN,
        GuiStatus::Failed => *RED,
    }
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        let (gui_comm, event_loop_comm) = Communicator::<UserAction, LoopMessage>::create_pair();
        let cloned_ctx = cc.egui_ctx.clone();
        let mut address = String::new();
        if let Some(storage) = cc.storage {
            if let Some(stored_address) = storage.get_string("address") {
                address = stored_address;
            }
        }
        start_event_loop(event_loop_comm, move || {
            cloned_ctx.request_repaint();
        });

        Self {
            address,
            comm: gui_comm,
            status: Default::default(),
            error_message: None,
        }
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "regular".to_owned(),
        egui::FontData::from_static(include_bytes!("assets/regular.ttf")),
    );
    fonts.font_data.insert(
        "bold".to_owned(),
        egui::FontData::from_static(include_bytes!("assets/bold.ttf")),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(FontFamily::Name(Arc::from("regular")))
        .or_default()
        .insert(0, "regular".to_owned());

    fonts
        .families
        .entry(FontFamily::Name(Arc::from("bold")))
        .or_default()
        .push("bold".to_owned());

    ctx.set_fonts(fonts);
    let mut style = (*ctx.style()).clone();
    let family_bold = FontFamily::Name(Arc::from("bold"));
    let family_regular = FontFamily::Name(Arc::from("regular"));
    style.text_styles = [
        (
            TextStyle::Button,
            FontId::new(18.0, family_regular.to_owned()),
        ),
        (
            TextStyle::Small,
            FontId::new(10.0, family_regular.to_owned()),
        ),
        (TextStyle::Heading, FontId::new(22.0, family_bold)),
        (TextStyle::Body, FontId::new(18.0, family_regular)),
    ]
    .into();
    ctx.set_style(style);
}
fn get_button_text(status: &GuiStatus) -> &str {
    if let GuiStatus::Connecting = status {
        return "Connect";
    }
    if status.can_connect() {
        return "Connect";
    }
    "Disconnect"
}

fn get_status_text(status: &GuiStatus) -> &str {
    match status {
        GuiStatus::Ready => "Waiting for connection",
        GuiStatus::Connecting => "Connecting...",
        GuiStatus::Connected => "Connected",
        GuiStatus::Failed => "Connection failed",
        GuiStatus::Disconnecting => "Disconnecting...",
        GuiStatus::Reconnecting => "Reconnecting...",
    }
}
