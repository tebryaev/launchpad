BINARY_NAME = launchpad
INSTALL_PATH = /usr/bin/$(BINARY_NAME)

.PHONY: all
all: build

.PHONY: run
run:
	cargo run

.PHONY: build
build:
	cargo build

.PHONY: release
release:
	cargo build --release

.PHONY: install
install: release
	sudo install -Dm755 target/release/$(BINARY_NAME) $(INSTALL_PATH)

.PHONY: uninstall
uninstall:
	sudo rm -f $(INSTALL_PATH)

.PHONY: clean
clean:
	cargo clean