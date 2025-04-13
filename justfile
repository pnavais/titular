alias b := build
alias full := build-full
alias min := build-min

build *args:
  cargo build {{args}}
build-min *args:
  cargo build --no-default-features --features minimal {{args}}
build-full *args:
  cargo build --features display {{args}}
release:
  cargo build --release
lint:
  cargo clippy -- -W clippy:all -W clippy::pedantic
lint-full:
  cargo clippy --no-default-features --features full_application -- -W clippy:all -W clippy::pedantic
reset:
  rm -fr $HOME/.config/titular
clean:
  cargo clean
full_clean: clean reset
