use std::path::Path;

use crate::crypto::KEY_LEN;
use crate::error::{Result, SottoError};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct KeyReport {
    pub backend: String,
    pub key_len: usize,
    pub fingerprint: String,
}

/// Load or create the 32-byte master key. macOS → Keychain; else file 0600.
pub fn load_or_create(_data_dir: &Path) -> Result<([u8; KEY_LEN], &'static str)> {
    Err(SottoError::app(
        "NOT_IMPLEMENTED",
        "KeyStore is not implemented in this wave",
        true,
        "Wait for harden GREEN.",
    ))
}
