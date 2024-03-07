use super::{
    bidirectional::{channel, Channel},
    codec::{LaserCodec, Reading},
};
use crossbeam_channel::select;
use futures::StreamExt;
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
    Read(Reading),
    Disconnected,
    Errored,
}

pub fn connect(
    ctx: egui::Context,
    port: &str,
) -> Result<Channel<Message, Event>, tokio_serial::Error> {
    match tokio_serial::new(port, BAUD_RATE).open_native_async() {
        Ok(serial) => {
            let (master, worker) = channel::<Message, Event>();

            tokio::spawn(async move {
                let mut framed = LaserCodec.framed(serial);
                let mut running = true;

                loop {
                    select! {
                        recv(worker.receiver) -> msg => {
                            match msg {
                                Ok(message) => match message {
                                    Message::Pause => running = false,
                                    Message::Resume => running = true,
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
                                        let _ = worker.send(Event::Read(reading));
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
        Err(e) => Err(e),
    }
}
