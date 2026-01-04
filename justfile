build:
	cargo +nightly build --release
fmt:
	cargo +nightly fmt
run:
	cargo +nightly run --release
test:
	cargo +nightly test -- --show-output
