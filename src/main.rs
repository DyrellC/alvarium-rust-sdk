use crypto::signatures::ed25519::{PublicKey, SecretKey};

fn main() {
    let priv_key = SecretKey::generate().unwrap();
    let pub_key = priv_key.public_key();

    println!("Priv: {}, Pub: {}", hex::encode(priv_key.as_slice()), hex::encode(pub_key.as_slice()));
}
