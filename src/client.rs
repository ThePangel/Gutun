use spake2::{Ed25519Group, Identity, Password, Spake2};

use crate::server::Enroll;

pub async fn start_client() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let body = reqwest::get("http://127.0.0.1:8080")
        .await?
        .error_for_status()?
        .text()
        .await?;

    println!("body = {body:?}");
    Ok(())
}

pub async fn pair(code: String) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let (s1, outbound_msg) = Spake2::<Ed25519Group>::start_a(
        &Password::new(code),
        &Identity::new(b"client"),
        &Identity::new(b"server"),
    );

    let response = Enroll {
        msg: outbound_msg,
        client_pubkey: "test".into(),
    };

    let inbound_msg: Vec<u8> = client
        .post("http://127.0.0.1:8080/api/enroll")
        .json(&response)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec();

    let key1 = s1.finish(&inbound_msg).unwrap();

    Ok(())
}
