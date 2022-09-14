use crate::RequestPaint;

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
