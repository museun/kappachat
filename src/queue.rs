use std::collections::VecDeque;

pub struct Queue<T> {
    max: usize,
    queue: VecDeque<T>,
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self::with_capacity(Self::DEFAULT_MAX)
    }
}

impl<T> Queue<T> {
    const DEFAULT_MAX: usize = 100;

    pub fn with_capacity(max: usize) -> Self {
        assert!(max != 0);
        Self {
            queue: VecDeque::with_capacity(max),
            max,
        }
    }

    pub fn push(&mut self, item: T) {
        while self.queue.len() >= self.max {
            self.queue.pop_front();
        }
        self.queue.push_back(item)
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> + ExactSizeIterator + DoubleEndedIterator {
        self.queue.iter()
    }
}
