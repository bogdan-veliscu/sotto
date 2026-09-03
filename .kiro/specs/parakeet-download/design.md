# Parakeet download — Design

Separate from `import_local` (REQ-MO-002). The desk offers explicit INT8 (~670 MB) and FP32 (~2.5 GB) buttons. Bytes land under app-data `models/parakeet-tdt-0.6b-v3/` after staging + layout check + rename, the same activate path as folder import.

Pinned source: `istupakov/parakeet-tdt-0.6b-v3-onnx` on Hugging Face. INT8 uses `encoder-model.int8.onnx` + `decoder_joint-model.int8.onnx` + `vocab.txt`. FP32 also copies `encoder-model.onnx.data`.

Production uses HTTPS. Tests inject a fetcher that writes tiny files and never opens a socket. `demo_pipeline` does not call download.

## Forbidden

- Auto-download on launch or Stop
- Fetching from `import_local`
- Unpinned / user-typed URLs
- Putting weights in git or fixtures
