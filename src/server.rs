use std::sync::Arc;

use crate::crypto::{Enroll, generate_server_hmac, verify_client_hmac};
use axum::{
    Json, Router,
    body::Bytes,
    extract::State,
    routing::{get, post},
};
use rand::{RngExt, distr::Alphanumeric};
use spake2::{Ed25519Group, Identity, Password, Spake2};
use tokio::sync::Mutex;

#[derive(Clone)]
struct AppState {
    password: String,
    key: Arc<Mutex<Vec<u8>>>,
}

pub async fn start_server(listen: String) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind(listen).await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}

pub async fn enroll(listen: String) -> Result<(), Box<dyn std::error::Error>> {
    let password: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .filter(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
        .take(5)
        .map(char::from)
        .collect();
    println!("Pairing code : {}", password);
    let app_state = AppState {
        password: password,
        key: Arc::new(Mutex::new(Vec::new())),
    };

    let app = Router::new()
        .route("/api/enroll", post(enroll_post))
        .route("/api/enroll/verify", post(verify_enroll))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(listen).await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}

async fn enroll_post(State(mut state): State<AppState>, body: Bytes) -> Vec<u8> {
    let (s1, outbound_msg) = Spake2::<Ed25519Group>::start_b(
        &Password::new(state.password),
        &Identity::new(b"client"),
        &Identity::new(b"server"),
    );

    let inbound_msg = body.to_vec();
    let mut key_lock = state.key.lock().await;
    *key_lock = s1.finish(&inbound_msg).unwrap();

    outbound_msg
}

async fn verify_enroll(State(state): State<AppState>, Json(enroll): Json<Enroll>) -> Json<Enroll> {
    let key = state.key.lock().await;

    if verify_client_hmac(&key, &enroll.hmac, &enroll.pub_key) {
        println!("Correctly saved clients keys");

        //TODO: Actually save client keys
    } else {
        panic!("Failed to pair to the client, please try again: Key verification failed")
    }

    Json(Enroll {
        hmac: generate_server_hmac(&key, &enroll.pub_key),
        pub_key: enroll.pub_key,
    })
}
