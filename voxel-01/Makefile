
build:
	cargo build

run: build
	cargo run


dev2:
	$(MAKE) run || true

dev:
	npx nodemon --watch src --ext rs --exec make -- dev2

