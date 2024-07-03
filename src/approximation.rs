use std::{collections::HashMap, sync::{Arc, Mutex}, time::Duration};

use chrono::{Duration as ChronoDuration, DurationRound, NaiveTime, TimeDelta, Timelike};
use tokio::time::Instant;

use crate::limiter::Limiter;

pub struct Approximation {
    pub duration: u64,
    pub threshold: usize,
    pub inner: Arc<Mutex<HashMap<String, Entry>>>,
    
}

pub struct Entry {
    pub pre_count: usize,
    pub cur_count: usize,
}

impl Limiter for Approximation {
    fn consume(&self, key: &str) -> anyhow::Result<usize, anyhow::Error> {
        let window = self.inner.lock().unwrap();

        let entry = window.get(key).unwrap();
        let now = Instant::now();
        
        Ok(0)
    }

    fn rearm(&self) -> Option<tokio::time::Instant> {
        todo!()
    }
}

// fn nearest_time() -> (Instant, Instant) {
//     use chrono::prelude::*;

//     let local: DateTime<Local> = Local::now();
//     local.duration_round(TimeDelta::minutes(1));
    
//     // 将持续时间转换为秒数
//     let total_seconds = since_epoch.as_secs();
    
//     // 计算当前时间在当天的秒数
//     let seconds_in_day = total_seconds % 86400;
    
//     // 将秒数转换为 NaiveTime
//     let current_time = NaiveTime::from_num_seconds_from_midnight_opt(seconds_in_day as u32, 0).unwrap();
    
//     // 计算当前时间的整分钟边界点
//     let minute_start_time = current_time.with_second(0).unwrap();
//     let minute_end_time = minute_start_time + ChronoDuration::minutes(1);
    
//     // 打印时间
//     println!("当前时间: {}", current_time);
//     println!("整分钟起始点: {}", minute_start_time);
//     println!("整分钟结束点: {}", minute_end_time);

//     // 计算边界点相对于当前时间点的偏移量
//     let minute_start_offset = (minute_start_time.num_seconds_from_midnight() as i64 - current_time.num_seconds_from_midnight() as i64) * 1_000_000_000;
//     let minute_end_offset = (minute_end_time.num_seconds_from_midnight() as i64 - current_time.num_seconds_from_midnight() as i64) * 1_000_000_000;

//     // 获取边界点的 Instant
//     let minute_start = now - Duration::from_nanos(minute_start_offset as u64);
//     let minute_end = now + Duration::from_nanos(minute_end_offset as u64);

//     (minute_start, minute_end)
// }

#[test]
fn test_nearest() {
    use chrono::prelude::*;

    let local: DateTime<Local> = Local::now();
    local.duration_round(TimeDelta::minutes(1));

}