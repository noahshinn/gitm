BINARY = gitm

BIN_DIR = /usr/local/bin

.PHONY: setup all

all:
	cargo build --release && mv target/release/$(BINARY) $(BIN_DIR);

setup:
	./setup

uninstall:
	rm $(BIN_DIR)/$(BINARY);
