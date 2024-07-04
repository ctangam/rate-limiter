pub mod bucket;
pub mod fixed;
pub mod limiter;
pub mod slide;
pub mod approximation;

use std::{collections::HashMap, sync::Arc};

use crate::limiter::Limiter;
use anyhow::{Error, Result};
use approximation::Approximation;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Router,
};
use fixed::FixedWindow;
use slide::SlidingWindow;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let shared_state = Approximation::init(60, 10);

    let app = Router::new()
        .route("/limited", get(limited))
        .route("/unlimited", get(unlimited))
        .with_state(shared_state);

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn limited(
    State(state): State<Arc<Approximation>>,
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
