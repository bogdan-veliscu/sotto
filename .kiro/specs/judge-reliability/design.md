# Judge reliability — Design

Restore the documented local judge commands before later waves depend on them.

## Test keystore

Add a narrowly scoped test/judge backend selected by a compile-time test path or an explicit Sotto-specific environment variable used only by Make targets. It stores a 0600 key inside the temporary data directory. Production `desktop` startup still requires Keychain and reports `keychain`; tests never fall back from a production Keychain error.

The contract proves repeated opens reuse the same test key, permissions remain 0600, production backend selection is unchanged, and no key bytes are printed.

## Judge boundary

`make demo`, `make contract`, and `make ci` set only the judge-keystore context. They do not enable fixture fallback for live paths, request permissions, start hardware capture, compile desktop-only decoders, or download weights. Demo remains the explicit CONSULT-001 fixture path.

## Forbidden

- Replacing production Keychain with a file backend
- Falling back from a production Keychain error
- Printing key material
- Prompting for TCC access or downloading models
