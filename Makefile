RUST_LOG ?= info

build:
	cargo +nightly build --release

test:
	cargo +nightly test --all-features

doc:
	cargo +nightly doc --all-features

benchmark:
	cargo +nightly build --release --all-features --examples
