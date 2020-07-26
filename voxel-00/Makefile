
build:
	cargo build --bin main

run: build
	cargo run --bin main


dev2:
	$(MAKE) run || true

dev:
	npx nodemon --watch src --ext rs --exec make -- dev2

archive:
	node scripts/archive_binary_version.js