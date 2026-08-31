# Model install — Design

Module: `src-tauri/src/install.rs`. Public API is locked by the contract tests. Do not rename.

```rust
pub const PARAKEET_ENGINE_ID: &str = "parakeet-tdt-0.6b-v3";

pub struct InstallResult {
    pub engine_id: String,
    pub bytes_written: u64,
    pub sha256: String,
}

pub fn parakeet_weights_path(cache_dir: &Path) -> PathBuf;
pub fn install_bytes(
    engine_id: &str,
    cache_dir: &Path,
    bytes: &[u8],
    expected_sha256: &str,
) -> Result<InstallResult>;
pub fn delete_model(engine_id: &str, cache_dir: &Path) -> Result<()>;
pub fn overlay_catalog(engines: Vec<Engine>, cache_dir: &Path) -> Vec<Engine>;
```

## Paths

- Parakeet: `cache_dir/models/parakeet-tdt-0.6b-v3.bin`
- Whisper (already): `cache_dir/models/ggml-large-v3-turbo.bin`

Never a URL as the destination.

## Checksum

SHA-256 hex, lowercase. Compare in constant time if easy; equality is acceptable. On mismatch: delete any temp/partial file, leave no dest file, `CHECKSUM_MISMATCH` recoverable.

Write via a sibling temp file then rename so a crash does not leave a "ready" truncated model.

`bytes_written` is progress for this PR (sync). No HTTP in `install_bytes`.

## Overlay

`list_engines` must run `overlay_catalog`. If the weights file exists, `install_state` is `ready`. If it does not, keep the catalog value (`not-installed`). Settings must stop lying once a file is on disk.

## Delete

Removes the weights file. Overlay then shows `not-installed`. Does not touch other engines. Does not download a replacement.

## Network

Do **not** add a download client that `demo_pipeline` or `make ci` can trigger. User-initiated fetch (pinned URL) may exist as a separate function that tests never call and demo never calls. If URL/source is unset, return `INSTALL_NO_SOURCE`. CI stays offline.

Do not invent a fake Hugging Face URL. Do not put 1.2 GB in git.

## Transcribe

Missing Parakeet file → `ENGINE_NOT_INSTALLED`. After a checksum-valid install, `transcribe_local` must not select cloud. A missing Parakeet runtime may return `ENGINE_NOT_BUILT` (honest). Do not replay the fixture as if it were Parakeet.

## Store / UI

`Store::list_engines` uses overlay with `data_dir`. Optional Tauri command `model_install_bytes` / `model_delete` for the desk. Settings copy should say install is user-initiated.

## Forbidden

- Auto-download on first launch or demo
- Silent cloud STT
- Keeping a file that failed checksum
- Editing `fixtures/` audio/transcripts
