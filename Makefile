EERO_IP ?=

.PHONY: lint test install test-live test-live-default

lint:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings

test: lint
	cargo test --workspace -- --skip live_

install:
	cargo install --path eeroctl

test-live:
	EERO_LIVE_TEST=1 EERO_IP=$(EERO_IP) cargo test -p eero-api --test live_tests

test-live-default:
	$(MAKE) test-live EERO_IP=$$(ip route show default | awk '/default/ {print $$3}')
