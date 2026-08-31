# Model install — Requirements

Lead-locked. Do not rewrite.

## REQ-IN-001: Checksum (INV-CHECKSUM)

**EARS:** WHEN `install_bytes` is given a payload whose SHA-256 does not match `expected_sha256`, THE SYSTEM SHALL return `CHECKSUM_MISMATCH`, SHALL NOT leave a weights file at the engine path, and SHALL NOT fetch a replacement from the network.

WHEN the SHA-256 matches, THE SYSTEM SHALL write the bytes to the engine path and return the digest.

CT-checksum.

## REQ-IN-002: Parakeet is local (INV-PARAKEET-LOCAL)

**EARS:** WHEN Parakeet weights are absent, `transcribe_local` for `parakeet-tdt-0.6b-v3` SHALL return `ENGINE_NOT_INSTALLED`. WHEN `install_bytes` has succeeded, `overlay_catalog` SHALL mark that engine `ready`, `transcribe_local` SHALL NOT return `CLOUD_DISABLED` or `ENGINE_NOT_INSTALLED`, and `delete_model` SHALL return it to not-installed.

CT-parakeet-local.

## REQ-IN-003: No silent cloud (INV-NO-SILENT-CLOUD)

**EARS:** WHEN the selected engine is unavailable, THE SYSTEM SHALL fall back only to a ready local engine, never cloud/api unless `cloud_mode` is on.

CT-no-silent-cloud (regression). Install never enables cloud.

Install is user-initiated. `demo_pipeline` SHALL NOT call `install_bytes` or any fetch.
