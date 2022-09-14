use std::collections::HashSet;

use flume::{Receiver, Sender};

use crate::helix::{Chatters, Client};

pub struct UserListUpdater {
    sub: Sender<Action>,
    wakeup: Sender<()>,
    receiver: Receiver<(String, Chatters)>,
}

impl UserListUpdater {
    pub fn create() -> Self {
        let (sub, subscribe) = flume::unbounded();
        let (sender, rx) = flume::unbounded();
        let (wakeup_tx, wakeup) = flume::unbounded();

        std::thread::spawn(move || {
            UserListUpdateInner {
                subscribe,
                sender,
                wakeup,
                set: <_>::default(),
            }
            .run()
        });

        Self {
            sub,
            receiver: rx,
            wakeup: wakeup_tx,
        }
    }

    pub fn poll(&self) -> Vec<(String, Chatters)> {
        let _ = self.wakeup.send(());
        self.receiver.try_iter().collect()
    }

    pub fn request_update(&mut self, channel: impl ToString) {
        let _ = self.sub.send(Action::Update(channel.to_string()));
    }

    pub fn subscribe(&mut self, channel: impl ToString) {
        let _ = self.sub.send(Action::Add(channel.to_string()));
    }

    pub fn unsubscribe(&mut self, channel: impl ToString) {
        let _ = self.sub.send(Action::Remove(channel.to_string()));
    }
}

enum Action {
    Add(String),
    Remove(String),
    Update(String),
}

struct UserListUpdateInner {
    subscribe: Receiver<Action>,
    sender: Sender<(String, Chatters)>,
    wakeup: Receiver<()>,
    set: HashSet<String>,
}

impl UserListUpdateInner {
    fn run(mut self) {
        let mut last = std::time::Instant::now();

        enum Either<L, R> {
            Left(L),
            Right(R),
        }

        loop {
            match flume::Selector::new()
                .recv(&self.subscribe, Either::Left)
                .recv(&self.wakeup, Either::Right)
                .wait()
            {
                Either::Left(Ok(action)) => self.handle_action(action),
                Either::Right(_) => {}
                _ => {}
            }

            if last.elapsed() > std::time::Duration::from_secs(10) {
                for channel in &self.set {
                    self.fetch(channel);
                }
                last = std::time::Instant::now();
            }
        }
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Add(ch) => {
                self.fetch(&ch);
                self.set.insert(ch);
            }
            Action::Remove(ch) => {
                self.set.remove(&ch);
            }
            Action::Update(ch) => {
                self.fetch(&ch);
            }
        }
    }

    fn fetch(&self, channel: &str) {
        let channel = channel.strip_prefix('#').unwrap_or(channel);

        match Client::get_chatters_for(channel) {
            Ok(chatters) => {
                let _ = self.sender.send((channel.to_string(), chatters));
            }
            Err(err) => {
                eprintln!("cannot get chatters for: {channel} because: {err}")
            }
        }
    }
}
