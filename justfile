set export
lint:
	cargo fmt --all && cargo clippy --all-targets -- -D warnings
test:
	#!/bin/bash
	cargo test -- --nocapture

build:
	#!/bin/bash
	set -e
	export RUSTFLAGS='-C link-arg=-s'
	cargo build --release --lib --target wasm32-unknown-unknown && rm -rf res && mkdir res && cp target/wasm32-unknown-unknown/release/nfthop.wasm res/

optimize:
	docker run --rm -v "$(pwd)":/code \
		--mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
		cosmwasm/rust-optimizer:0.12.11

checksum:
	#!/bin/bash
	cat res/checksums.txt | grep -e nfthop.wasm -e > checksum

all: lint build test