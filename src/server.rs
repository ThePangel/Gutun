use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use clap::builder::Str;
use rand::{RngExt, distr::Alphanumeric};
use serde::Deserialize;
use spake2::{Ed25519Group, Identity, Password, Spake2};
use tokio::sync::Mutex;

#[derive(Deserialize)]
struct enroll {
    msg: Vec<u8>,
    client_pubkey: String,
}

#[derive(Clone, Default)]
struct AppState {
    pub spake_session: Arc<Mutex<Option<Spake2<Ed25519Group>>>>,
    pub derived_key: Arc<Mutex<Option<Vec<u8>>>>,
}

pub async fn start_server(listen: String) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState {
        spake_session: Arc::new(Mutex::new(None)),
        derived_key: Arc::new(Mutex::new(None)),
    };
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route(
            "/api/enroll",
            get(enroll_get).post(enroll_post).with_state(state),
        );

    let listener = tokio::net::TcpListener::bind(listen).await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}

async fn enroll_get(State(state): State<AppState>) -> Vec<u8> {
    let password: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .filter(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
        .take(5)
        .map(char::from)
        .collect();

    println!("Pairing code : {}", password);

    let (s1, outbound_msg) = Spake2::<Ed25519Group>::start_b(
        &Password::new(password),
        &Identity::new(b"client"),
        &Identity::new(b"server"),
    );

    let mut pake = state.spake_session.lock().await;
    *pake = Some(s1);

    outbound_msg
}

async fn enroll_post(State(state): State<AppState>, Json(payload): Json<enroll>) {
    let mut session_lock = state.spake_session.lock().await;
    let s1 = match session_lock.take() {
        Some(s1) => s1,
        None => panic!("Session not founf or lost"),
    };

    let inbound_msg = payload.msg;
    let key2 = s1.finish(&inbound_msg).unwrap();
}
