default: all

all: lint build test

test:
	$(MAKE) -C ../factory build || break;
	$(MAKE) -C ../pool build || break;
	cargo test

build:
	$(MAKE) -C ../factory build || break;
	$(MAKE) -C ../pool build || break;
	cargo build --target wasm32-unknown-unknown --release

lint: fmt clippy

fmt:
	cargo fmt --all

clippy: build
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean
