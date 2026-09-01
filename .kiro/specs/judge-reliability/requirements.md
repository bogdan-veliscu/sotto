# Judge reliability — Requirements

Lead-locked. Do not rewrite.

## REQ-JR-001: Deterministic test keystore (INV-KEYCHAIN-TEST-DETERMINISTIC)

**EARS:** WHEN `make demo`, `make contract`, or `make ci` runs in a test or judge context on macOS, THE SYSTEM SHALL use an isolated deterministic test keystore without opening a Keychain prompt, while production desktop builds SHALL continue to use macOS Keychain. Test isolation SHALL NOT weaken the production key backend or expose key bytes.

CT-keychain-test-deterministic.

## REQ-JR-002: Judge commands complete without desktop resources (INV-JUDGE-COMPLETES)

**EARS:** WHEN `make demo`, `make contract`, or `make ci` runs after dependencies are cached, THE SYSTEM SHALL complete without microphone, Screen Recording, model weights, or desktop UI and SHALL keep the demo on `fixture-replay` with zero network calls.

CT-judge-completes.

This spec does not certify the desktop, hardware, TCC, or real-model path.
