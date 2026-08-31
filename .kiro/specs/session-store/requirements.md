# Session store — Requirements

Lead-locked. Do not rewrite.

## REQ-ST-001: Private defaults (INV-NO-CLOUD-DEFAULT)

**EARS:** WHEN the local store is created, THE SYSTEM SHALL set `cloud_mode` to `off` and `telemetry` to `off`.

CT-no-cloud-default.

## REQ-ST-002: Fixture lock (INV-FIXTURE-LOCK)

**EARS:** WHEN fixtures are validated, THE SYSTEM SHALL verify SHA-256 digests against `harness/graph/fixture-lock.json`.

CT-fixtures-lock.

## REQ-ST-003: Engine policy (INV-NO-SILENT-CLOUD)

**EARS:** WHEN the selected engine is unavailable, THE SYSTEM SHALL fall back only to a ready local engine and SHALL NOT select cloud or api unless `cloud_mode` is `on`.

CT-no-silent-cloud.
