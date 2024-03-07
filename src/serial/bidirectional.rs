use crossbeam_channel::{Receiver, RecvError, SendError, Sender, TryRecvError};

#[derive(Debug)]
pub struct Channel<S, R> {
    pub(crate) sender: Sender<S>,
    pub(crate) receiver: Receiver<R>,
}

impl<S, R> Channel<S, R> {
    pub fn send(&self, s: S) -> Result<(), SendError<S>> {
        self.sender.send(s)
    }

    pub fn recv(&self) -> Result<R, RecvError> {
        self.receiver.recv()
    }

    pub fn try_recv(&self) -> Result<R, TryRecvError> {
        self.receiver.try_recv()
    }
}

pub fn channel<S, R>() -> (Channel<S, R>, Channel<R, S>) {
    let (ls, lr) = crossbeam_channel::unbounded();
    let (rs, rr) = crossbeam_channel::unbounded();

    (
        Channel {
            sender: ls,
            receiver: rr,
        },
        Channel {
            sender: rs,
            receiver: lr,
        },
    )
}
