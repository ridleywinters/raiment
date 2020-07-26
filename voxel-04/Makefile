
build:
	cargo build

run: build
	RUST_BACKTRACE=1 cargo run


dev2:
	$(MAKE) run || true

dev:
	npx nodemon --watch src --ext rs --exec make -- dev2

format:
	cargo fmt
