use spake2::{Ed25519Group, Identity, Password, Spake2};

use crate::crypto::{Enroll, generate_client_hmac, verify_server_hmac};

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

    let inbound_msg: Vec<u8> = client
        .post("http://127.0.0.1:8080/api/enroll")
        .body(outbound_msg)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec();

    let key1 = s1.finish(&inbound_msg).unwrap();

    let enroll = Enroll {
        hmac: generate_client_hmac(&key1, &b"test".to_vec()),
        pub_key: b"test".to_vec(),
    };

    let hmac_veryfing: Enroll = client
        .post("http://127.0.0.1:8080/api/enroll/verify")
        .json(&enroll)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    if verify_server_hmac(&key1, &hmac_veryfing.hmac, &b"test".to_vec()) {
        println!("Suscessfully paired to server");

        //TODO: Save keys and ip into a config file
        
    } else {
        panic!("Failed to pair to the server, please try again: Key verification failed")
    }
    Ok(())
}
