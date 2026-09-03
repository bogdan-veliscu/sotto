# Parakeet download — Requirements

## REQ-PD-001: User-initiated TDT download (INV-PARAKEET-DOWNLOAD)

**EARS:** WHEN the user starts a Parakeet download from the desk, THE SYSTEM SHALL fetch the pinned Hugging Face TDT files for the chosen INT8 or FP32 pack into a staging directory, validate the layout, and activate it atomically. A failed download SHALL leave the previous TDT directory unchanged. `import_local` SHALL still refuse URLs. `make demo` and contract tests SHALL NOT download weights and SHALL keep `network_calls` at 0.

CT-parakeet-download.

No auto-download. No weights in git. No silent cloud STT.
