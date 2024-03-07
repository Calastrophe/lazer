use eframe::egui;
use egui_notify::Toasts;
use serial::{available_ports, Event, Message, Serial};
mod serial;

#[derive(Default)]
pub struct Laser {
    connection: Option<Serial>,
    toasts: Toasts,
}

impl eframe::App for Laser {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if let Some(serial) = &mut self.connection {
            match serial.channel.recv() {
                Ok(event) => match event {
                    Event::Read(reading) => serial.update(reading),
                    Event::Disconnected => {
                        self.toasts.info("Successfully disconnected!");
                        self.connection = None;
                    }
                    Event::Errored => {
                        self.toasts.info("An error occurred when reading!");
                        self.connection = None;
                    }
                },
                Err(_) => {
                    self.toasts
                        .error("The worker dropped without telling us...");
                    self.connection = None;
                }
            }
        }

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if let Some(serial) = &self.connection {
                    if ui
                        .image(egui::include_image!("../assets/disconnect.png"))
                        .clicked()
                    {
                        let _ = serial.channel.send(Message::Disconnect);
                    };

                    ui.separator();

                    if ui
                        .image(egui::include_image!("../assets/pause.png"))
                        .clicked()
                    {
                        let _ = serial.channel.send(Message::Pause);
                    };

                    if ui
                        .image(egui::include_image!("../assets/resume.png"))
                        .clicked()
                    {
                        let _ = serial.channel.send(Message::Resume);
                    };
                } else {
                    ui.menu_image_button(egui::include_image!("../assets/ports.png"), |ui| {
                        available_ports().map(|ports| {
                            ports.iter().for_each(|port| {
                                if ui.button(port).clicked() {
                                    match Serial::new(ctx.clone(), port) {
                                        Ok(conn) => self.connection = Some(conn),
                                        Err(e) => {
                                            tracing::info!("failed to connect: {e}");
                                        }
                                    }
                                }
                            });
                        });
                    });
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(serial) = &self.connection {
                serial.show(ui);
            }
        });

        self.toasts.show(ctx);
    }
}

fn main() -> eframe::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("failed to set subscriber");

    eframe::run_native(
        "lazer",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(Laser::default())
        }),
    )
}
