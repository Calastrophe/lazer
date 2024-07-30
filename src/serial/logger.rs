use super::codec::Reading;
use crossbeam_channel::{unbounded, Sender};
use csv::Writer;
use std::{
    fs::File,
    io::Error,
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::warn;

pub enum Command {
    Write(Reading),
    NewFile,
    Kill,
}

pub fn logger(path: String) -> Result<Sender<Command>, Error> {
    let (tx, rx) = unbounded();

    let mut log = create_log(&path)?;

    tokio::spawn(async move {
        loop {
            match rx.recv() {
                Ok(msg) => match msg {
                    Command::Write(reading) => {
                        let _ = log.serialize(reading);
                    }
                    Command::NewFile => {
                        log = match create_log(&path) {
                            Ok(log) => log,
                            Err(e) => {
                                warn!("Killing logging thread due to: {e}");

                                return;
                            }
                        }
                    }
                    Command::Kill => return,
                },
                Err(e) => {
                    warn!("Killing logging thread due to: {e}");
                    return;
                }
            }
        }
    });

    Ok(tx)
}

fn create_log(path: &str) -> Result<Writer<File>, Error> {
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(Writer::from_writer(File::create(format!(
        "{path}/log-{epoch}.csv"
    ))?))
}
