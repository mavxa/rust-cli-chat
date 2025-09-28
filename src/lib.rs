// src/lib.rs
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn send_frame(
    stream: &mut (impl AsyncWriteExt + Unpin),
    payload: &[u8],
) -> Result<()> {
    let len = (payload.len() as u32).to_be_bytes();
    stream.write_all(&len).await?;
    stream.write_all(payload).await?;
    Ok(())
}

pub async fn recv_frame(
    stream: &mut (impl AsyncReadExt + Unpin),
) -> Result<Vec<u8>> {
    let mut lenb = [0u8; 4];
    stream.read_exact(&mut lenb).await?;
    let len = u32::from_be_bytes(lenb) as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    Ok(buf)
}

/* ---------- crypto helpers ---------- */
use std::convert::TryInto;
use x25519_dalek::{PublicKey, StaticSecret};
use hkdf::Hkdf;
use sha2::Sha256;
use chacha20poly1305::{
    XChaCha20Poly1305,
    aead::{Aead, KeyInit},
    XNonce,
};
use rand::rngs::OsRng;
use rand::RngCore;

/// Генерация пары X25519 (ephemeral)
pub fn gen_x25519_keypair() -> (StaticSecret, PublicKey) {
    let mut rng = OsRng;
    let sk = StaticSecret::random_from_rng(&mut rng);
    let pk = PublicKey::from(&sk);
    (sk, pk)
}

/// shared secret (32 bytes)
pub fn shared_secret_bytes(sk: &StaticSecret, peer_pk: &PublicKey) -> [u8; 32] {
    let ss = sk.diffie_hellman(peer_pk);
    *ss.as_bytes()
}

/// derive 32-byte key from shared secret via HKDF-SHA256
pub fn derive_key_from_shared(shared: &[u8]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(None, shared);
    let mut okm = [0u8; 32];
    hk.expand(b"rust-cli-e2e-chat", &mut okm).expect("hkdf expand ok");
    okm
}

pub fn aead_from_key(key: &[u8; 32]) -> XChaCha20Poly1305 {
    XChaCha20Poly1305::new(key.into())
}

pub fn encrypt_message(aead: &XChaCha20Poly1305, plaintext: &[u8]) -> Vec<u8> {
    // 24-byte nonce
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);
    let ct = aead.encrypt(XNonce::from_slice(&nonce), plaintext).expect("encrypt");
    let mut out = Vec::with_capacity(24 + ct.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ct);
    out
}

pub fn decrypt_message(aead: &XChaCha20Poly1305, payload: &[u8]) -> Result<Vec<u8>> {
    if payload.len() < 24 { anyhow::bail!("payload too short"); }
    let (nonce, ct) = payload.split_at(24);
    let pt = aead.decrypt(XNonce::from_slice(nonce), ct)
        .map_err(|e| anyhow::anyhow!("decrypt failed: {:?}", e))?;
    Ok(pt)
}

/// helper: convert Vec<u8> length-32 into PublicKey
pub fn pubkey_from_bytes(v: &[u8]) -> Result<PublicKey> {
    let arr: [u8; 32] = v.try_into().map_err(|_| anyhow::anyhow!("pubkey len != 32"))?;
    Ok(PublicKey::from(arr))
}
