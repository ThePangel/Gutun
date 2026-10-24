use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(Serialize, Deserialize, Debug)]
pub struct Enroll {
    pub hmac: Vec<u8>,
    pub pub_key: Vec<u8>,
}

pub fn generate_hmac(key: &[u8], label: &[u8], message: &[u8]) -> Vec<u8> {
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(&key).expect("HMAC key initialization failed");
    mac.update(label);
    mac.update(&message);

    let result = mac.finalize();

    result.into_bytes().to_vec()
}

pub fn verify_hmac(key: &[u8], label: &[u8], hmac: &[u8], message: &[u8]) -> bool {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(&key).expect("HMAC verification failed");
    mac.update(label);
    mac.update(&message);

    mac.verify_slice(&hmac[..]).is_ok()
}

pub fn verify_client_hmac(key: &[u8], hmac: &[u8], message: &[u8]) -> bool {
    verify_hmac(key, b"client", hmac, message)
}

pub fn verify_server_hmac(key: &[u8], hmac: &[u8], message: &[u8]) -> bool {
    verify_hmac(key, b"server", hmac, message)
}

pub fn generate_client_hmac(key: &[u8], message: &[u8]) -> Vec<u8> {
    generate_hmac(key, b"client", message)
}

pub fn generate_server_hmac(key: &[u8], message: &[u8]) -> Vec<u8> {
    generate_hmac(key, b"server", message)
}
