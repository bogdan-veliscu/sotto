# Speech-to-text pipeline

Turn recorded audio into timestamped segments locally.

## Default engines

Parakeet TDT 0.6B v3 (planned). Whisper Large-v3 Turbo (planned). Fixture replay (wave 1).

## Stages

Normalize → language detect → STT → punctuation → segment → store → FTS.

## v1 vs v2

v1 is batch after stop. v2 may stream with progress events.

## Quality

Keep timestamps. Keep the transcript editable. Never silently rewrite meaning. Keep raw and cleaned separate.

## Fallback

Retry once. Then next **local** configured model. Record failure reason on the session. Never silent cloud.
