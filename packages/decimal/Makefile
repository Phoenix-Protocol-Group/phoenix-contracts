default: all

all: lint build test

test:
	cargo test

build:
	soroban contract build

lint: fmt clippy

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean
