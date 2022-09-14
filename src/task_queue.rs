use crate::{
    helix::{self, Chatters},
    RequestPaint,
};

pub struct TaskQueue<I> {
    queue: flume::Sender<I>,
    ready: flume::Receiver<(I, Vec<u8>)>,
    handle: std::thread::JoinHandle<()>,
}

impl<I> TaskQueue<I>
where
    I: Send + Sync + 'static,
{
    pub fn new<R, F>(repaint: R, spawn: F) -> Self
    where
        R: RequestPaint + 'static,
        F: FnOnce(R, flume::Receiver<I>, flume::Sender<(I, Vec<u8>)>) + Send + Sync + 'static,
    {
        let (queue_tx, queue_rx) = flume::unbounded();
        let (ready_tx, ready_rx) = flume::unbounded();

        let handle = std::thread::spawn(move || spawn(repaint, queue_rx, ready_tx));

        Self {
            queue: queue_tx,
            ready: ready_rx,
            handle,
        }
    }

    pub fn join(self) -> Vec<(I, Vec<u8>)> {
        drop(self.queue);
        let _ = self.handle.join();
        self.ready.into_iter().collect()
    }

    pub fn enqueue(&self, item: I) {
        let _ = self.queue.send(item);
    }

    pub fn try_next(&self) -> Option<(I, Vec<u8>)> {
        self.ready.try_recv().ok()
    }
}

pub struct UserListUpdate {
    sub: flume::Sender<Action>,
    wakeup: flume::Sender<()>,
    receiver: flume::Receiver<((usize, String), crate::helix::Chatters)>,
}

impl UserListUpdate {
    pub fn new() -> Self {
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

    pub fn poll(&self) -> Vec<((usize, String), Chatters)> {
        let _ = self.wakeup.send(());
        self.receiver.try_iter().collect()
    }

    pub fn request_update(&mut self, room_id: usize) {
        let _ = self.sub.send(Action::Update(room_id));
    }

    pub fn subscribe(&mut self, room_id: usize, channel: impl ToString) {
        let _ = self.sub.send(Action::Add((room_id, channel.to_string())));
    }

    pub fn unsubscribe(&mut self, room_id: usize, channel: impl ToString) {
        let _ = self
            .sub
            .send(Action::Remove((room_id, channel.to_string())));
    }
}

enum Action {
    Add((usize, String)),
    Remove((usize, String)),
    Update(usize),
}

struct UserListUpdateInner {
    subscribe: flume::Receiver<Action>,
    sender: flume::Sender<((usize, String), crate::helix::Chatters)>,
    wakeup: flume::Receiver<()>,
    set: std::collections::HashSet<(usize, String)>,
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
                .recv(&self.subscribe, |t| Either::Left(t))
                .recv(&self.wakeup, |_| Either::Right(()))
                .wait()
            {
                Either::Left(Ok(action)) => self.handle_action(action),
                Either::Right(_) => {}
                _ => {}
            }

            if last.elapsed() > std::time::Duration::from_secs(10) {
                for (id, channel) in &self.set {
                    self.fetch(*id, channel);
                }
                last = std::time::Instant::now();
            }
        }
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Add((id, ch)) => {
                self.fetch(id, &ch);
                self.set.insert((id, ch));
            }
            Action::Remove(ch) => {
                self.set.remove(&ch);
            }
            Action::Update(id) => {
                if let Some(ch) = self.set.iter().find_map(|(l, ch)| (*l == id).then_some(ch)) {
                    self.fetch(id, &ch);
                }
            }
        }
    }

    fn fetch(&self, room_id: usize, mut channel: &str) {
        if channel.starts_with('#') {
            channel = &channel[1..]
        }

        match helix::Client::get_chatters_for(channel) {
            Ok(chatters) => {
                let _ = self.sender.send(((room_id, channel.to_string()), chatters));
            }
            Err(err) => {
                eprintln!("cannot get chatters for: {channel} because: {err}")
            }
        }
    }
}
