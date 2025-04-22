alias b := build
alias full := build-full
alias min := build-min

build *args:
  cargo build {{args}}
build-full *args:
  cargo build --features display {{args}}
build-min *args:
  cargo build --no-default-features --features minimal {{args}}
clean:
  cargo clean
full_clean: clean reset
lint:
  cargo clippy -- -W clippy:all -W clippy::pedantic
lint-full:
  cargo clippy --no-default-features --features full_application -- -W clippy:all -W clippy::pedantic
release:
  cargo build --release
reset:
  rm -fr $HOME/.config/titular
test:
  cargo test
