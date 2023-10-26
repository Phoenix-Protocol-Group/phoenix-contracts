ifeq (,$(BUILD_FLAGS))
  DEPS = ../stake ../pool
endif

default: all

all: lint build test

test: build # because of token dependency
	$(MAKE) -C ../multihop build || break;
	cargo test

build:
	@for dir in $(DEPS) ; do \
		$(MAKE) -C $$dir build || break; \
	done
	cargo build --target wasm32-unknown-unknown --release

lint: fmt clippy

fmt:
	cargo fmt --all

clippy: build
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean
