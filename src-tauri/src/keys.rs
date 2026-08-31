use std::fs;
use std::path::Path;

use crate::crypto::{self, KEY_LEN};
use crate::error::{Result, SottoError};

pub const KEY_FILE: &str = "master.key";

#[cfg(target_os = "macos")]
const SERVICE: &str = "com.bogdanveliscu.sotto";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct KeyReport {
    pub backend: String,
    pub key_len: usize,
    pub fingerprint: String,
}

pub fn fingerprint(key: &[u8; KEY_LEN]) -> String {
    crypto::sha256_hex(key).chars().take(16).collect()
}

/// Load or create the 32-byte master key. macOS → Keychain; else file 0600.
pub fn load_or_create(data_dir: &Path) -> Result<([u8; KEY_LEN], &'static str)> {
    #[cfg(target_os = "macos")]
    {
        macos_load_or_create(data_dir)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok((file_load_or_create(data_dir)?, "file"))
    }
}

pub fn file_load_or_create(data_dir: &Path) -> Result<[u8; KEY_LEN]> {
    fs::create_dir_all(data_dir)?;
    let path = data_dir.join(KEY_FILE);
    if path.exists() {
        let bytes = fs::read(&path)?;
        if bytes.len() != KEY_LEN {
            return Err(SottoError::app(
                "KEY_INVALID",
                "Local master key is the wrong length.",
                false,
                "Do not replace master.key. Restore it with the audio files.",
            ));
        }
        restrict_file(&path)?;
        let mut key = [0u8; KEY_LEN];
        key.copy_from_slice(&bytes);
        Ok(key)
    } else {
        let key = crypto::new_master_key();
        fs::write(&path, key)?;
        restrict_file(&path)?;
        Ok(key)
    }
}

fn restrict_file(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn account_for(data_dir: &Path) -> String {
    crypto::sha256_hex(data_dir.to_string_lossy().as_bytes())
}

#[cfg(target_os = "macos")]
fn macos_load_or_create(data_dir: &Path) -> Result<([u8; KEY_LEN], &'static str)> {
    use security_framework::passwords::{get_generic_password, set_generic_password};

    fs::create_dir_all(data_dir)?;
    let account = account_for(data_dir);
    if let Ok(bytes) = get_generic_password(SERVICE, &account) {
        if bytes.len() == KEY_LEN {
            let mut key = [0u8; KEY_LEN];
            key.copy_from_slice(&bytes);
            let leftover = data_dir.join(KEY_FILE);
            if leftover.exists() {
                let _ = fs::remove_file(leftover);
            }
            return Ok((key, "keychain"));
        }
    }

    let key = if data_dir.join(KEY_FILE).exists() {
        file_load_or_create(data_dir)?
    } else {
        crypto::new_master_key()
    };
    set_generic_password(SERVICE, &account, &key).map_err(|err| {
        SottoError::app(
            "KEYCHAIN",
            format!("Could not store the master key in Keychain: {err}"),
            true,
            "Unlock Keychain and open Sotto again.",
        )
    })?;
    let leftover = data_dir.join(KEY_FILE);
    if leftover.exists() {
        let _ = fs::remove_file(leftover);
    }
    Ok((key, "keychain"))
}
