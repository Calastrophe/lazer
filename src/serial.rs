use crossbeam_channel::TryRecvError;
use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use bidirectional::Channel;
use codec::Reading;
use egui_plot::{Line, Plot, PlotPoints};
pub use reader::{Event, Message};

pub const MAX_SAMPLES: usize = 60;

mod bidirectional;
mod codec;
mod logger;
mod reader;

#[derive(Debug, PartialEq)]
pub enum Plotting {
    Reference,
    Measured,
    Displacement,
    Velocity,
}

pub struct Serial {
    pub(crate) channel: Channel<Message, Event>,
    pub(crate) selected_plot: Plotting,
    readings: Arc<RwLock<VecDeque<Reading>>>,
}

impl Serial {
    /// Attempts to open the given serial port, returning an error if it fails to connect.
    pub fn new(ctx: egui::Context, port: &str, path: String) -> Result<Self, std::io::Error> {
        let readings = Arc::new(RwLock::new(VecDeque::with_capacity(MAX_SAMPLES)));

        Ok(Self {
            channel: reader::connect(ctx, port, path, readings.clone())?,
            selected_plot: Plotting::Measured,
            readings,
        })
    }

    pub fn is_disconnected(&self, toasts: &mut egui_notify::Toasts) -> bool {
        match self.channel.try_recv() {
            Ok(event) => match event {
                Event::Disconnected => {
                    toasts.info("Successfully disconnected!");
                    true
                }
                Event::Errored => {
                    toasts.error("An error occurred when reading!");
                    true
                }
            },
            Err(TryRecvError::Disconnected) => {
                toasts.error("The worker thread dropped without telling us...");
                true
            }
            Err(_) => false,
        }
    }

    /// Show a plot of the current serial readings
    pub fn show(&self, ui: &mut egui::Ui) {
        let points: PlotPoints = self
            .readings
            .read()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, r)| match self.selected_plot {
                Plotting::Reference => [i as f64, r.reference as f64],
                Plotting::Measured => [i as f64, r.measured as f64],
                Plotting::Velocity => [i as f64, r.velocity as f64],
                Plotting::Displacement => [i as f64, r.displacement],
            })
            .collect();

        Plot::new("serial_plot")
            .view_aspect(5.0)
            .show_x(false)
            .show_grid(false)
            .show_axes([false, true])
            .show(ui, |plot_ui| plot_ui.line(Line::new(points)));
    }
}

pub fn available_ports() -> Option<Vec<String>> {
    Some(
        tokio_serial::available_ports()
            .ok()?
            .into_iter()
            .map(|p| p.port_name)
            .collect(),
    )
}
