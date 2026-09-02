# Apple Speech — Design

Use Apple's on-device SpeechAnalyzer / SpeechTranscriber on macOS 26+. Audio is a local WAV file. `requiresOnDeviceRecognition` / offline transcriber presets only. First use may install Apple's on-device language assets through `AssetInventory`; that is an OS model install, not Sotto uploading meeting audio.

`live_ready` is true only when `SpeechTranscriber.isAvailable`. Overlay does not download assets and does not prompt TCC. Transcription may prompt Speech Recognition permission.

Linux / `--no-default-features` compiles a stub: `ENGINE_NOT_BUILT`, `live_ready=false`.

`demo_pipeline` never selects this engine. Contract tests never call the recognizer.

## Forbidden

- Server-side `SFSpeechRecognizer` (default Apple cloud path)
- Using Apple Speech in `make demo`
- Claiming live-ready on Linux
