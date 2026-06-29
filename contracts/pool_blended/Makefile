default: all

all: lint build test

test: build # because of token dependency
	cargo test

build:
	$(MAKE) -C ../token build || break;
	$(MAKE) -C ../stake build || break;
	cargo build --target wasm32-unknown-unknown --release

lint: fmt clippy

fmt:
	cargo fmt --all

clippy: build
	cargo clippy --all-targets -- -D warnings -A clippy::too_many_arguments

clean:
	cargo clean
