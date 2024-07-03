use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration},
};

use anyhow::{Error, Result};
use tokio::{select, sync::Notify, time::{self, Instant}};

use crate::limiter::Limiter;

pub struct FixedWindow {
    pub duration: u64,
    pub threshold: usize,
    pub inner: Arc<Mutex<HashMap<String, (Instant, usize)>>>,
    pub background_task: Notify,
}

impl FixedWindow {
    pub fn init(duration: u64, threshold: usize) -> Arc<Self> {
        let window = Self { duration, threshold, inner: Arc::new(Mutex::new(HashMap::new())), background_task: Notify::new() };
    
        let shared_state = Arc::new(window);
        let state = shared_state.clone();
        tokio::spawn(purge_expired_tasks(state));

        shared_state
    }
}

async fn purge_expired_tasks(state: Arc<FixedWindow>) {
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

impl Limiter for FixedWindow {
    fn consume(&self, key: &str) -> Result<usize, Error> {
        let mut window = self.inner.lock().unwrap();
        let (_, token) = *window.entry(key.to_string()).and_modify(|(_, token)| {
            if *token > 0 {
                *token -= 1;
                println!("{key} token used: {token}")
            }
        }).or_insert((Instant::now() + Duration::from_secs(self.duration), self.threshold - 1));

        if token <= 0 {
            return Err(Error::msg("tokens run out"));
        }

        self.background_task.notify_one();

        Ok(token)
    }

    fn rearm(&self) -> Option<Instant> {
        let mut window = self.inner.lock().unwrap();
        let mut next = None;
        let now = Instant::now();
        window.retain(|_, (when, _)| {
            if next.is_none() || next.is_some_and(|n| n > *when) {
                next = Some(*when);
            }
            *when > now
        });

        println!("window refreshed");

        next
    }
}