use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};
use tokio::sync::{Mutex, Notify};

#[derive(Clone)]
pub struct DownloadProgress {
    pub total_bytes: Arc<AtomicU64>,
    pub downloaded_bytes: Arc<AtomicU64>,
    pub start_time: Instant,
}

impl DownloadProgress {
    pub fn downloaded(&self) -> u64 {
        self.downloaded_bytes.load(Ordering::SeqCst)
    }
}

#[derive(Clone)]
pub struct ProgressSlotPool {
    bars: Arc<Vec<ProgressBar>>,
    available: Arc<Mutex<VecDeque<usize>>>,
    notify: Arc<Notify>,
}

impl ProgressSlotPool {
    pub fn new(bars: Vec<ProgressBar>) -> Self {
        let mut queue = VecDeque::with_capacity(bars.len());
        for idx in 0..bars.len() {
            queue.push_back(idx);
        }

        Self {
            bars: Arc::new(bars),
            available: Arc::new(Mutex::new(queue)),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn acquire_slot(&self) -> usize {
        loop {
            if let Some(idx) = {
                let mut guard = self.available.lock().await;
                guard.pop_front()
            } {
                return idx;
            }

            self.notify.notified().await;
        }
    }

    pub async fn release_slot(&self, idx: usize) {
        {
            let mut guard = self.available.lock().await;
            guard.push_back(idx);
        }
        self.notify.notify_one();
    }

    pub fn bar(&self, idx: usize) -> ProgressBar {
        self.bars[idx].clone()
    }

    pub fn len(&self) -> usize {
        self.bars.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bars.is_empty()
    }
}

#[derive(Clone)]
pub struct ProgressDisplay {
    pub total_bar: ProgressBar,
    pub slot_pool: ProgressSlotPool,
    _multi: Arc<MultiProgress>,
}

impl ProgressDisplay {
    pub fn new(concurrency: usize, total_size: u64) -> Self {
        let multi = Arc::new(MultiProgress::new());

        let total_bar = multi.add(ProgressBar::new(total_size));
        total_bar
            .set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [TOTAL] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta}, {binary_bytes_per_sec})")
                    .unwrap()
                    .progress_chars("#>-"),
            );

        let mut bars = Vec::with_capacity(concurrency);
        for idx in 0..concurrency {
            let bar = multi.add(ProgressBar::new(0));
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{prefix}] [{wide_bar:.yellow/blue}] {bytes}/{total_bytes} ({eta}, {binary_bytes_per_sec}) {msg}")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            bar.set_prefix(format!("TASK {:02}", idx + 1));
            bar.set_message("idle");
            bars.push(bar);
        }

        Self {
            total_bar,
            slot_pool: ProgressSlotPool::new(bars),
            _multi: multi,
        }
    }
}
