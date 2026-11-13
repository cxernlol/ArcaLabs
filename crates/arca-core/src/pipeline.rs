use std::{
    collections::VecDeque,
    sync::{
        Condvar, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

pub struct BoundedQueue<T> {
    capacity: usize,
    values: Mutex<VecDeque<T>>,
    available: Condvar,
    closed: AtomicBool,
    dropped: AtomicU64,
}

impl<T> BoundedQueue<T> {
    pub fn new(capacity: usize) -> Result<Self, &'static str> {
        if capacity == 0 {
            return Err("Queue capacity must be greater than zero");
        }
        Ok(Self {
            capacity,
            values: Mutex::new(VecDeque::with_capacity(capacity)),
            available: Condvar::new(),
            closed: AtomicBool::new(false),
            dropped: AtomicU64::new(0),
        })
    }

    pub fn push_latest(&self, value: T) -> Result<(), T> {
        if self.closed.load(Ordering::Acquire) {
            return Err(value);
        }
        let Ok(mut values) = self.values.lock() else {
            return Err(value);
        };
        if self.closed.load(Ordering::Acquire) {
            return Err(value);
        }
        if values.len() == self.capacity {
            values.pop_front();
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
        values.push_back(value);
        self.available.notify_one();
        Ok(())
    }

    pub fn pop_timeout(&self, timeout: Duration) -> Option<T> {
        let values = self.values.lock().ok()?;
        let (mut values, _) = self
            .available
            .wait_timeout_while(values, timeout, |values| {
                values.is_empty() && !self.closed.load(Ordering::Acquire)
            })
            .ok()?;
        values.pop_front()
    }

    pub fn close(&self) {
        self.closed.store(true, Ordering::Release);
        if let Ok(_values) = self.values.lock() {
            self.available.notify_all();
        }
    }
    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }
}

#[derive(Default)]
pub struct PipelineMetrics {
    captured: AtomicU64,
    encoded: AtomicU64,
    dropped: AtomicU64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricsSnapshot {
    pub captured: u64,
    pub encoded: u64,
    pub dropped: u64,
}
impl PipelineMetrics {
    pub fn frame_captured(&self) {
        self.captured.fetch_add(1, Ordering::Relaxed);
    }
    pub fn frame_encoded(&self) {
        self.encoded.fetch_add(1, Ordering::Relaxed);
    }
    pub fn frame_dropped(&self) {
        self.dropped.fetch_add(1, Ordering::Relaxed);
    }
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            captured: self.captured.load(Ordering::Relaxed),
            encoded: self.encoded.load(Ordering::Relaxed),
            dropped: self.dropped.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_full_queue_discards_oldest_frame() {
        let queue = BoundedQueue::new(2).unwrap();
        queue.push_latest(1).unwrap();
        queue.push_latest(2).unwrap();
        queue.push_latest(3).unwrap();
        assert_eq!(queue.pop_timeout(Duration::ZERO), Some(2));
        assert_eq!(queue.pop_timeout(Duration::ZERO), Some(3));
        assert_eq!(queue.dropped(), 1);
    }
    #[test]
    fn a_closed_queue_rejects_new_work() {
        let queue = BoundedQueue::new(1).unwrap();
        queue.close();
        assert_eq!(queue.push_latest(7), Err(7));
        assert_eq!(queue.pop_timeout(Duration::ZERO), None);
    }
}
