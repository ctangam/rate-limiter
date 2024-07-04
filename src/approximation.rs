use std::{
    collections::{BTreeSet, HashMap},
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{Error, Result};
use chrono::{prelude::*, DurationRound, TimeDelta};
use tokio::time::{self, Instant};

use crate::limiter::Limiter;

pub struct Entry {
    pub hits: BTreeSet<DateTime<Local>>,
}

pub struct Approximation {
    pub duration: u32,
    pub threshold: usize,
    pub inner: Arc<Mutex<HashMap<String, Entry>>>,
}

impl Approximation {
    pub fn init(duration: u32, threshold: usize) -> Arc<Self> {
        let window = Self {
            duration,
            threshold,
            inner: Arc::new(Mutex::new(HashMap::new())),
        };

        let shared_state = Arc::new(window);
        let state = shared_state.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(time::Duration::from_secs(state.duration as u64));
            loop {
                interval.tick().await;

                println!("purge outdated hits");
                state.rearm();
            }
        });

        shared_state
    }
}

impl Limiter for Approximation {
    fn consume(&self, key: &str) -> Result<usize, Error> {
        let mut window = self.inner.lock().unwrap();
        let entry = window.entry(key.to_string()).or_insert(Entry {
            hits: BTreeSet::new(),
        });

        let now = Local::now();
        let left = now
            .checked_sub_signed(TimeDelta::seconds(self.duration as i64))
            .unwrap();
        let (prev_start, prev_end) = nearest_minute(left);
        let prev_weight = 1.0 - now.second() as f64 / self.duration as f64;
        let mut prev_count = 0;
        let mut cur_count = 0;
        for hit in &entry.hits {
            if prev_start <= *hit && *hit < prev_end {
                prev_count += 1;
            } else if *hit >= prev_end {
                cur_count += 1;
            }
        }
        println!("prev_weight: {prev_weight}, prev_count: {prev_count}, cur_count: {cur_count}");
        
        let approximation = prev_count as f64 * prev_weight + cur_count as f64;
        let remained = self.threshold - approximation as usize;
        println!("calc hits: {approximation}, remained: {remained}");
        
        if remained <= 0 {
            return Err(Error::msg("No more tokens"));
        }

        entry.hits.insert(now);

        Ok(remained - 1)
    }

    fn rearm(&self) -> Option<tokio::time::Instant> {
        let mut window = self.inner.lock().unwrap();
        let now = Local::now();
        let left = now
            .checked_sub_signed(TimeDelta::seconds(self.duration as i64))
            .unwrap();
        println!("ddl: {left}");
        window.retain(|key, entry| {
            println!("{key} before clean: {:?}", entry.hits);
            entry.hits.retain(|hit| *hit >= left);
            println!("{key} after clean: {:?}", entry.hits);

            !entry.hits.is_empty()
        });

        None
    }
}

fn nearest_minute(when: DateTime<Local>) -> (DateTime<Local>, DateTime<Local>) {
    let minute_start = when.duration_trunc(TimeDelta::minutes(1)).unwrap();
    let minute_end = minute_start
        .checked_add_signed(TimeDelta::minutes(1))
        .unwrap();

    (minute_start, minute_end)
}
