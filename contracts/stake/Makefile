default: all

all: lint build test

test: build
	cargo test

build:
	$(MAKE) -C ../token build || break;
	RUSTFLAGS="-C target-cpu=mvp" cargo build --target wasm32-unknown-unknown --release

lint: fmt clippy

fmt:
	cargo fmt --all

clippy: build
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean
