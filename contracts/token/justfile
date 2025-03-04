export RUSTFLAGS := "-C strip=none"

target := "target"

build:
    cargo build --release --target=wasm32-unknown-unknown --features certora

clean:
    rm -rf {{target}}
