default: build

all: test

test: build
	cargo test

build:
	cargo build --target wasm32-unknown-unknown --release

fmt:
	cargo fmt --all

clippy: build
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean
