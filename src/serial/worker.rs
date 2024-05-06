use super::{
    bidirectional::{channel, Channel},
    codec::{LaserCodec, Reading},
    MAX_SAMPLES,
};
use crossbeam_channel::select;
use csv::Writer;
use futures::StreamExt;
use std::{
    collections::VecDeque,
    fs::File,
    io,
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Decoder;
use tracing::info;

const BAUD_RATE: u32 = 2000000;

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
    data: Arc<RwLock<VecDeque<Reading>>>,
) -> Result<Channel<Message, Event>, io::Error> {
    let serial = tokio_serial::new(port, BAUD_RATE).open_native_async()?;

    let mut log = create_log(&path)?;

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
                                log = match create_log(&path) {
                                    Ok(log) => log,
                                    Err(e) => {
                                        let _ = worker.send(Event::Errored);

                                        info!("Killing worker thread: {e}");
                                        ctx.request_repaint();

                                        return;
                                    }
                                };
                                running = true;
                            },
                            Message::Disconnect => {
                                let _ = worker.send(Event::Disconnected);
                                ctx.request_repaint();

                                return;
                            }
                        },
                        Err(e) => {
                            info!("Killing worker thread: {e}");
                            return;
                        },
                    }
                }
                default() => {
                    if !running {
                        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                        continue;
                    }

                    if let Some(reading) = framed.next().await {
                        match reading {
                            Ok(reading) => {
                                {
                                    let mut data = data.write().unwrap();

                                    if data.len() >= MAX_SAMPLES {
                                        data.pop_front();
                                    }

                                    data.push_back(reading);
                                }

                                let _  = log.serialize(reading);

                                ctx.request_repaint();
                            }
                            Err(e) => {
                                let _ = worker.send(Event::Errored);

                                info!("Killing worker thread: {e}");
                                ctx.request_repaint();

                                return;
                            }
                        }
                    }


                }
            }
        }
    });

    Ok(master)
}

fn create_log(path: &str) -> Result<Writer<File>, io::Error> {
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(Writer::from_writer(File::create(format!(
        "{path}/log-{epoch}.csv"
    ))?))
}
