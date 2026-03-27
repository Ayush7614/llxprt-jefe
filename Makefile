.PHONY: build ci-check quick-check

build: ci-check

ci-check:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets --all-features -- -D warnings
	cargo clippy --workspace --all-targets --all-features -- \
		-A clippy::all \
		-A clippy::pedantic \
		-A clippy::nursery \
		-D clippy::cognitive_complexity \
		-D clippy::too_many_lines \
		-D clippy::too_many_arguments \
		-D clippy::type_complexity \
		-D clippy::struct_excessive_bools
	cargo llvm-cov \
		--workspace \
		--all-features \
		--summary-only \
		--ignore-filename-regex '(/vendor/|/tmp/|/rustc-)' \
		--fail-under-lines 30
	cargo build --workspace --all-features --locked
	cargo test --workspace --all-features --locked

quick-check:
	cargo fmt
	cargo check -q
	cargo test -q
