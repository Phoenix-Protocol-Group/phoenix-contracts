default: all

all: lint build test

test: build # because of token dependency
	cargo test

build:
	$(MAKE) -C ../token build || break;
	soroban contract build

lint: fmt clippy

fmt:
	cargo fmt --all

clippy: build
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean
