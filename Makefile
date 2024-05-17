.PHONY: test
test:
	export RUSTFLAGS=-Awarnings
	cargo test