use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::error::{Result, SottoError};

pub const NONCE_LEN: usize = 12;
pub const KEY_LEN: usize = 32;

pub fn new_master_key() -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

pub fn encrypt(key: &[u8; KEY_LEN], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext).map_err(|_| {
        SottoError::app(
            "ENCRYPT_FAILED",
            "Could not encrypt audio.",
            true,
            "Retry. The plaintext temp file is deleted on failure.",
        )
    })?;
    let mut packed = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    packed.extend_from_slice(&nonce_bytes);
    packed.extend_from_slice(&ciphertext);
    Ok(packed)
}

pub fn decrypt(key: &[u8; KEY_LEN], packed: &[u8]) -> Result<Vec<u8>> {
    if packed.len() < NONCE_LEN {
        return Err(SottoError::app(
            "DECRYPT_TRUNCATED",
            "Encrypted audio is truncated.",
            false,
            "The file cannot be recovered. Re-record if you still have the meeting.",
        ));
    }
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(&packed[..NONCE_LEN]);
    cipher.decrypt(nonce, &packed[NONCE_LEN..]).map_err(|_| {
        SottoError::app(
            "DECRYPT_FAILED",
            "Could not decrypt audio with the local master key.",
            false,
            "The key file and the audio file must stay together.",
        )
    })
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub fn looks_like_wav(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WAVE"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_not_wav() {
        let key = new_master_key();
        let wav = b"RIFF\x24\x00\x00\x00WAVEfmt ";
        let packed = encrypt(&key, wav).unwrap();
        assert!(!looks_like_wav(&packed));
        assert_eq!(decrypt(&key, &packed).unwrap(), wav);
    }
}
