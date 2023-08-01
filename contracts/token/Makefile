default: build

all: test

test: build
	cargo test

build:
	cargo build --target wasm32-unknown-unknown --release

clippy:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt --all

clean:
	cargo clean
