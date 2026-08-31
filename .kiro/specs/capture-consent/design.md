# Capture consent — Design

Consent is a column on `sessions`. The UI shows a paper disclosure card before `recorder_begin`.

`finalize_with_wav` AES-GCM-encrypts bytes, writes `audio/*.sotto`, stores checksum.
