
build:
	cargo build

run: build
	cargo run --release

run-debug: build
	RUST_BACKTRACE=1 cargo run

dev2:
	$(MAKE) run-debug || true

dev:
	npx nodemon --watch src --ext rs,png --exec make -- dev2

format:
	cargo fmt
