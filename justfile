set working-directory := "codex-rs"
set positional-arguments

# Display help
help:
    just -l

# `esv`
alias c := esv
esv *args:
    cargo run --bin esv -- "$@"

fmt:
    cargo fmt -- --config imports_granularity=Item

fix *args:
    cargo clippy --fix --all-features --tests --allow-dirty "$@"

clippy:
    cargo clippy --all-features --tests "$@"

install:
    rustup show active-toolchain
    cargo fetch
