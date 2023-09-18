test:
	cargo test -- --nocapture
.PHONY: test

build: deps
	UPDATE_SUBMODULES=true cargo build
.PHONY: build

deps:
	sudo apt update && sudo apt install cmake build-essential curl libcurl4-openssl-dev libssl-dev uuid-dev libclang-dev pkg-config -y
.PHONY: deps
