use std::{collections::{BTreeSet, HashMap}, sync::{Arc, Mutex}, time::{Duration}};

use anyhow::{Result, Error};
use tokio::{select, sync::Notify, time::{self, Instant}};

use crate::limiter::Limiter;

pub struct SlidingWindow {
    pub duration: u64,
    pub threshold: usize,
    pub inner: Arc<Mutex<HashMap<String, BTreeSet<Instant>>>>,
    pub background_task: Notify,
}

impl SlidingWindow {
    pub fn init(duration: u64, threshold: usize) -> Arc<Self> {
        let window = Self {
            duration,
            threshold,
            inner: Arc::new(Mutex::new(HashMap::new())),
            background_task: Notify::new()
        };

        let shared_state = Arc::new(window);
        let state = shared_state.clone();
        tokio::spawn(purge_expired_tasks(state));

        shared_state
    }
}

async fn purge_expired_tasks(state: Arc<SlidingWindow>) {
    loop {
        if let Some(when) = state.rearm() {
            println!("sleep until {}", when.duration_since(Instant::now()).as_secs());
            select! {
                _ = time::sleep_until(when) => {}
                _ = state.background_task.notified() => {}
            }
        } else {
            state.background_task.notified().await;
        }
    }
}

impl Limiter for SlidingWindow {
    fn consume(&self, key: &str) -> Result<usize, Error> {
        let mut window = self.inner.lock().unwrap();
        let when = Instant::now() + Duration::from_secs(self.duration);
        let set = window.entry(key.to_string()).and_modify(|s| {
            if s.len() < self.threshold {
                s.insert(when);
                self.background_task.notify_one();
                println!("Req added, remaining: {}", self.threshold - s.len());
            }
        }).or_insert_with(|| {
            let mut s = BTreeSet::new();
            s.insert(when);
            self.background_task.notify_one();
            println!("New req added, remaining: {}", self.threshold - 1);
            s
        });

        let remained = self.threshold - set.len();

        if remained <= 0 {
            return Err(Error::msg("tokens run out"));
        }

        Ok(remained)
    }

    fn rearm(&self) -> Option<Instant> {
        let mut window = self.inner.lock().unwrap();
        let mut next = None;
        let now = Instant::now();
        window.iter_mut().for_each(|(key, s)| {
            while let Some(&when) = s.iter().next() {
                if when > now {
                    if next.is_none() || next.is_some_and(|n| n > when) {
                        next = Some(when);
                    }
                    break;
                }


                s.remove(&when);
                println!("{key}:{:?} expired", when);
            }
        });
        window.retain(|_, s| !s.is_empty());

        next
    }
}