.DEFAULT_GOAL := build

.PHONY: build
build: target/doc
	cargo build --all-features

target/doc: Cargo.* src/**
	cargo doc --all-features

.PHONY: lint
lint:
	cargo +nightly clippy -- -Wclippy::pedantic

.PHONY: test
test:
	cargo test

.PHONY: lint
lint:
	cargo +nightly clippy --all-targets --all-features -- -Wclippy::pedantic
	cargo fmt --check

.PHONY: fix
fix:
	cargo +nightly clippy --fix --allow-staged --all-targets --all-features -- -Dclippy::pedantic
	cargo fmt

