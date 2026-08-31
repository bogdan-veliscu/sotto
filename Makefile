.PHONY: graph verify contract demo lint test dev check lock

UV ?= uv
PYTEST = python3 -m pytest
CARGO = cargo
NPM = npm

graph:
	python3 harness/scripts/validate_graph.py

lock:
	SOTTO_ALLOW_FIXTURE_MUTATION=1 python3 harness/scripts/validate_graph.py --rewrite-lock

lint:
	cd src-tauri && $(CARGO) fmt --check
	$(NPM) run check

verify: graph
	cd src-tauri && $(CARGO) test --lib
	$(NPM) run check

contract: graph
	cd src-tauri && $(CARGO) test --test contract -- --nocapture
	$(PYTEST) tests/contract -q --tb=short

test: verify contract

demo:
	cd src-tauri && $(CARGO) run --quiet --bin sotto-demo

dev:
	$(NPM) run tauri dev

check: graph contract
	@echo "sotto ok"

status:
	python3 harness/scripts/validate_graph.py --status
