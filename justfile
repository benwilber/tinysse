build:
    cargo build
    
release:
    cargo build --release

clean:
    cargo clean

fmt:
    cargo fmt
    
check:
    cargo fmt --check
    cargo clippy
    
fix:
    cargo clippy --fix
    
fixdirty:
    cargo clippy --fix --allow-dirty
    
test:
    cargo test

doc:
    cargo doc --no-deps

opendoc:
    cargo doc --no-deps --open

ready: fmt check test doc

run *args:
    cargo run -- {{ args }}

runrelease *args:
    cargo run --release -- {{ args }}

