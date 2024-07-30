use super::{
    bidirectional::{channel, Channel},
    codec::{LaserCodec, Reading},
    logger::{logger, Command},
    MAX_SAMPLES,
};
use crossbeam_channel::select;
use futures::StreamExt;
use std::{
    collections::VecDeque,
    io,
    sync::{Arc, RwLock},
};
use tokio::time::{sleep, Duration, Instant};
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Decoder;
use tracing::{debug, warn};

const BAUD_RATE: u32 = 2000000;
const DISPLACEMENT: f64 = 79.2;

pub enum Message {
    Disconnect,
    Pause,
    Resume,
}

pub enum Event {
    Disconnected,
    Errored,
}

pub fn connect(
    ctx: egui::Context,
    port: &str,
    path: String,
    readings: Arc<RwLock<VecDeque<Reading>>>,
) -> Result<Channel<Message, Event>, io::Error> {
    let serial = tokio_serial::new(port, BAUD_RATE).open_native_async()?;

    let tx = logger(path)?;

    let (master, worker) = channel();

    tokio::spawn(async move {
        let mut framed = LaserCodec.framed(serial);
        let mut running = true;

        loop {
            select! {
                recv(worker.receiver) -> msg => {
                    match msg {
                        Ok(message) => match message {
                            Message::Pause => running = false,
                            Message::Resume => {
                                let _ = tx.send(Command::NewFile);

                                running = true;
                            },
                            Message::Disconnect => {
                                let _ = tx.send(Command::Kill);
                                let _ = worker.send(Event::Disconnected);
                                ctx.request_repaint();

                                return;
                            }
                        },
                        Err(e) => {
                            warn!("Killing worker thread due to: {e}");
                            ctx.request_repaint();

                            return;
                        },
                    }
                }
                default() => {
                    if !running {
                        sleep(Duration::from_millis(250)).await;
                        continue;
                    }

                    let start = Instant::now();

                    if let Some(reading) = framed.next().await {
                        match reading {
                            Ok(mut reading) => {

                                {
                                    let readings = readings.read().unwrap();

                                    let prev_total_displacement = readings.back().map_or(0, |r| r.total_displacement);

                                    // The 'as' will totally not backfire if the number grows large enough...
                                    reading.displacement = (reading.total_displacement - prev_total_displacement) as f64 * DISPLACEMENT;
                                }

                                let _ = tx.try_send(Command::Write(reading));

                                {
                                    let mut readings = readings.write().unwrap();

                                    if readings.len() >= MAX_SAMPLES {
                                        readings.pop_front();
                                    }

                                    readings.push_back(reading);
                                }

                                ctx.request_repaint();
                            }
                            Err(e) => {
                                let _ = worker.send(Event::Errored);

                                warn!("Killing worker thread due to: {e}");
                                ctx.request_repaint();

                                return;
                            }
                        }
                    }

                    let duration = start.elapsed();
                    debug!("The entire read took: {duration:?}");

                }
            }
        }
    });

    Ok(master)
}
