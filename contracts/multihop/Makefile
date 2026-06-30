ifeq (,$(BUILD_FLAGS))
  DEPS = ../factory ../pool ../pool_blended
endif

default: all

all: lint build test

test:
	$(MAKE) -C ../factory build || break;
	$(MAKE) -C ../pool build || break;
	$(MAKE) -C ../pool_blended build || break;
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
