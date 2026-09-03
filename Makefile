.PHONY: graph verify contract demo lint test dev check lock ci build cert-desktop

PYTEST = python3 -m pytest
CARGO = cargo
NPM = npm
MANIFEST = --manifest-path src-tauri/Cargo.toml
CORE = --no-default-features
JUDGE_KEYSTORE = SOTTO_JUDGE_KEYSTORE=isolated-file

graph:
	python3 harness/scripts/validate_graph.py

lock:
	SOTTO_ALLOW_FIXTURE_MUTATION=1 python3 harness/scripts/validate_graph.py --rewrite-lock

lint:
	cd src-tauri && $(CARGO) fmt --check
	$(NPM) run check

verify: graph
	$(JUDGE_KEYSTORE) $(CARGO) test $(MANIFEST) $(CORE) --lib
	$(NPM) run check

contract: graph
	$(JUDGE_KEYSTORE) $(CARGO) test $(MANIFEST) $(CORE) --lib --test contract -- --nocapture
	$(PYTEST) tests/contract -q --tb=short

test: verify contract

demo:
	$(JUDGE_KEYSTORE) $(CARGO) run $(MANIFEST) $(CORE) --quiet --bin sotto-demo

dev:
	$(NPM) run tauri dev

# macOS desk .app (skips flaky DMG). Binary is `sotto`, not sotto-demo.
build:
	$(NPM) run tauri -- build --bundles app

# Same gates as .github/workflows/ci.yml (minus npm ci).
ci: graph
	$(JUDGE_KEYSTORE) $(CARGO) test $(MANIFEST) $(CORE) --lib --test contract
	$(JUDGE_KEYSTORE) $(CARGO) run $(MANIFEST) $(CORE) --quiet --bin sotto-demo
	$(PYTEST) tests/contract -q --tb=short
	$(NPM) run check
	@echo "sotto ci ok"

check: ci

# macOS founder certification. Never requests TCC. Never stores transcript/audio.
cert-desktop:
	python3 harness/scripts/macos_cert.py --automated
