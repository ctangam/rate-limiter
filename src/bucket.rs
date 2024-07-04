use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration},
};

use anyhow::{Error, Result};
use tokio::time::{self, Instant};

use crate::limiter::Limiter;

pub struct Bucket {
    pub capacity: usize,
    pub inner: Arc<Mutex<HashMap<String, usize>>>,
}

impl Bucket {
    pub fn init(capacity: usize) -> Arc<Bucket> {
        let bucket = Self {
            capacity,
            inner: Arc::new(Mutex::new(HashMap::new())),
        };

        let shared_state = Arc::new(bucket);
        let state = shared_state.clone();
        tokio::spawn(async move {
            loop {
                if let Some(when) = state.rearm() {
                    println!("sleep until {}", when.duration_since(Instant::now()).as_secs());
                    time::sleep_until(when).await;
                }
            }
        });

        shared_state
    }
}

impl Limiter for Bucket {
    fn consume(&self, key: &str) -> Result<usize, Error> {
        let mut bucket = self.inner.lock().unwrap();
        let token = *bucket
            .entry(key.to_string())
            .and_modify(|token| {
                if *token > 0 {
                    *token -= 1;
                    println!("{key} token used: {token}")
                }
            })
            .or_insert(self.capacity - 1);

        if token <= 0 {
            return Err(Error::msg("tokens run out"));
        }

        Ok(token)
    }

    fn rearm(&self) -> Option<Instant> {
        let mut bucket = self.inner.lock().unwrap();
        bucket.iter_mut().for_each(|(key, token)| {
            if *token < self.capacity {
                *token += 1;
                println!("{key} token added: {token}");
            }
        });

        None
    }
}
