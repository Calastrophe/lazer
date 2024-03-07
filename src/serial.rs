use csv::Writer;
use std::collections::VecDeque;
use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};

use bidirectional::Channel;
use codec::Reading;
use egui_plot::{Line, Plot, PlotPoints};
pub use worker::{Event, Message};

mod bidirectional;
mod codec;
mod worker;

pub enum Plotting {
    Reference,
    Measured,
    Displacement,
    Velocity,
}

pub struct Serial {
    pub(crate) channel: Channel<Message, Event>,
    plotting: Plotting,
    file: Writer<File>,
    data: VecDeque<Reading>,
}

impl Serial {
    /// Attempts to open the given serial port, returning an error if it fails to connect.
    pub fn new(ctx: egui::Context, port: &str) -> Result<Self, std::io::Error> {
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let file = File::create(format!("log-{epoch}.csv"))?;

        Ok(Self {
            file: Writer::from_writer(file),
            plotting: Plotting::Measured,
            channel: worker::connect(ctx, port)?,
            data: VecDeque::with_capacity(60),
        })
    }

    /// Appends a new value to the plot, getting rid of one if at 60.
    pub fn update(&mut self, reading: Reading) {
        if self.data.len() >= 60 {
            self.data.pop_back();
        }

        // Ignore if it fails...
        let _ = self.file.serialize(reading);

        self.data.push_front(reading)
    }

    pub fn set_plot(&mut self, plot: Plotting) {
        self.plotting = plot;
    }

    /// Show a plot of the current serial readings
    pub fn show(&self, ui: &mut egui::Ui) {
        let points: PlotPoints = self
            .data
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let v = match self.plotting {
                    Plotting::Reference => r.reference,
                    Plotting::Measured => r.measured,
                    Plotting::Velocity => r.velocity,
                    Plotting::Displacement => r.displacement,
                };

                [i as f64, v as f64]
            })
            .collect();

        Plot::new("serial_plot")
            .show_x(false)
            .show_grid(false)
            .view_aspect(2.0)
            .show(ui, |plot_ui| plot_ui.line(Line::new(points)));
    }
}

pub fn available_ports() -> Option<Vec<String>> {
    Some(
        tokio_serial::available_ports()
            .ok()?
            .iter()
            .map(|p| p.port_name.clone())
            .collect(),
    )
}
