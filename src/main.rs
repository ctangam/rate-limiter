use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use anyhow::{Error, Result};
use axum::{
    extract::{ConnectInfo, Host, Query, Request, State}, http::StatusCode, response::IntoResponse, routing::get, Extension, Router
};
use tokio::{
    net::TcpListener,
    time::{self, Instant},
};

pub trait Storage {
    fn consume(&self, key: &str) -> Result<(), Error>;

    fn rearm(&self);
}

pub struct Bucket {
    pub capacity: usize,
    pub inner: Arc<Mutex<HashMap<String, usize>>>,
}

impl Bucket {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Storage for Bucket {
    fn consume(&self, key: &str) -> Result<(), Error> {
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

        Ok(())
    }

    fn rearm(&self) {
        let mut bucket = self.inner.lock().unwrap();
        bucket.iter_mut().for_each(|(key, token)| {
            if *token <= self.capacity {
                *token += 1;
                println!("{key} token added: {token}");
            }
        });
    }
}

pub struct FixedWindow {
    pub duration: u64,
    pub threshold: usize,
    pub inner: Arc<Mutex<HashMap<String, (Instant, usize)>>>
}

#[tokio::main]
async fn main() {
    let storage = Bucket::new(11);
    let shared_state = Arc::new(storage);

    let state = shared_state.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            println!("rearm tokens");
            state.rearm();
        }
    });

    let app = Router::new()
        .route("/limited", get(limited))
        .route("/unlimited", get(unlimited))
        .with_state(shared_state);

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn limited(
    State(state): State<Arc<Bucket>>,
    Query(params): Query<HashMap<String, String>>,
) -> (StatusCode, &'static str) {
    let key = params.get("id").unwrap();
    if let Ok(_) = state.consume(key) {
        (StatusCode::OK, "Limited, don't over use me!")
    } else {
        (StatusCode::TOO_MANY_REQUESTS, "no more tokens")
    }
}

async fn unlimited() -> &'static str {
    "Unlimited! Let's Go!"
}
