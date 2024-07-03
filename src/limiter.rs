
use anyhow::{Result, Error};
use tokio::time::Instant;

pub trait Limiter {
    fn consume(&self, key: &str) -> Result<usize, Error>;

    fn rearm(&self) -> Option<Instant>;
}