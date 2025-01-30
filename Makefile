all: test format-check lint

demo:
	cargo run --example demo --features chrono-tz

test:
	@cargo test
	@cargo test --all-features

format:
	@rustup component add rustfmt 2> /dev/null
	@cargo fmt --all

format-check:
	@rustup component add rustfmt 2> /dev/null
	@cargo fmt --all -- --check

lint:
	@rustup component add clippy 2> /dev/null
	@cargo clippy

.PHONY: all test format format-check lint demo
