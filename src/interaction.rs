use flume::{Receiver, Sender};

use crate::twitch::{Message, Twitch};

pub struct Interaction {
    sender_tx: Sender<String>,
    sender_rx: Receiver<String>,

    receiver_tx: Sender<Message>,
    receiver_rx: Receiver<Message>,
}

impl Default for Interaction {
    fn default() -> Self {
        Self::create()
    }
}

impl Interaction {
    pub fn create() -> Self {
        let (sender_tx, sender_rx) = flume::bounded(16);
        let (receiver_tx, receiver_rx) = flume::unbounded();

        Self {
            sender_tx,
            sender_rx,
            receiver_tx,
            receiver_rx,
        }
    }

    pub fn poll(&self, twitch: &Twitch) -> anyhow::Result<()> {
        twitch.poll(&self.sender_rx, &self.receiver_tx)
    }

    pub fn try_read(&self) -> Option<Message> {
        self.receiver_rx.try_recv().ok()
    }

    pub fn send_raw(&self, msg: impl ToString) {
        let _ = self.sender_tx.send(msg.to_string());
    }
}
