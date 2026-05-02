alias b := build
alias full := build-full
alias min := build-min
alias c := check

build *args:
  cargo build {{args}}
build-full *args:
  cargo build --features display {{args}}
build-min *args:
  cargo build --no-default-features --features minimal {{args}}
full-app *args:
  cargo build --no-default-features --features full_application {{args}}
clean:
  cargo clean
full_clean: clean reset
# Apply rustfmt to the whole workspace (writes files).
fmt:
  cargo fmt --all
lint:
  cargo clippy -- -W clippy::all -W clippy::pedantic
# Fails if anything needs formatting; does not write files (use `just fmt` to fix). Then clippy + tests.
check:
  cargo fmt --all -- --check && cargo clippy -- -W clippy::all -W clippy::pedantic && cargo test
lint-full:
  cargo clippy --no-default-features --features full_application -- -W clippy::all -W clippy::pedantic
# Same feature set and Clippy flags as lint-full; use this instead of `cargo clippy --fix --lib` alone.
lint-full-fix:
  cargo clippy --no-default-features --features full_application --fix --lib -p titular --allow-dirty --allow-staged -- -W clippy::all -W clippy::pedantic
release:
  cargo build --release
reset:
  rm -fr $HOME/.config/titular
test *args:
  cargo test {{args}}
