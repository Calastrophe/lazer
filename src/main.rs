use eframe::egui;
use egui::Button;
use egui_notify::Toasts;
use serial::{available_ports, Message, Plotting, Serial};
mod serial;

#[derive(Default)]
pub struct Laser {
    connection: Option<Serial>,
    toasts: Toasts,
}

impl eframe::App for Laser {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if let Some(serial) = &mut self.connection {
            if serial.is_disconnected(&mut self.toasts) {
                self.connection = None;
            }
        }

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if let Some(serial) = &mut self.connection {
                    if ui
                        .add(Button::image(egui::include_image!(
                            "../assets/disconnect.png"
                        )))
                        .clicked()
                    {
                        let _ = serial.channel.send(Message::Disconnect);
                    };

                    ui.separator();

                    if ui
                        .add(Button::image(egui::include_image!("../assets/pause.png")))
                        .clicked()
                    {
                        let _ = serial.channel.send(Message::Pause);
                    };

                    if ui
                        .add(Button::image(egui::include_image!("../assets/resume.png")))
                        .clicked()
                    {
                        let _ = serial.channel.send(Message::Resume);
                    };

                    egui::ComboBox::from_label("")
                        .selected_text(format!("{:?}", serial.plotting))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut serial.plotting,
                                Plotting::Reference,
                                "Reference",
                            );
                            ui.selectable_value(
                                &mut serial.plotting,
                                Plotting::Measured,
                                "Measured",
                            );
                            ui.selectable_value(
                                &mut serial.plotting,
                                Plotting::Velocity,
                                "Velocity",
                            );
                            ui.selectable_value(
                                &mut serial.plotting,
                                Plotting::Displacement,
                                "Displacement",
                            );
                        });
                } else {
                    ui.menu_image_button(egui::include_image!("../assets/ports.png"), |ui| {
                        available_ports().map(|ports| {
                            ports.iter().for_each(|port| {
                                if ui.button(port).clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                        match Serial::new(
                                            ctx.clone(),
                                            port,
                                            path.to_string_lossy().to_string(),
                                        ) {
                                            Ok(conn) => self.connection = Some(conn),
                                            Err(e) => {
                                                tracing::info!("failed to connect: {e}");
                                            }
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

#[tokio::main]
async fn main() -> eframe::Result<()> {
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
