
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use rand::{RngExt, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use spake2::{Ed25519Group, Identity, Password, Spake2};
#[derive(Deserialize, Serialize)]
pub struct Enroll {
    pub msg: Vec<u8>,
    pub client_pubkey: String,
}
#[derive(Clone)]
struct AppState {
    password: String,
    key: Vec<u8>
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
    let app_state = AppState { password: password, key: Vec::new() };

    let app = Router::new().route("/api/enroll", post(enroll_post).with_state(app_state));

    let listener = tokio::net::TcpListener::bind(listen).await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}

async fn enroll_post(State(mut state): State<AppState>, Json(payload): Json<Enroll>) -> Vec<u8> {
    let (s1, outbound_msg) = Spake2::<Ed25519Group>::start_b(
        &Password::new(state.password),
        &Identity::new(b"client"),
        &Identity::new(b"server"),
    );

    let inbound_msg = payload.msg;
    state.key = s1.finish(&inbound_msg).unwrap();

    outbound_msg
}
