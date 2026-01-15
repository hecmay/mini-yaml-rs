.PHONY: build build-web build-bundler clean test publish link help

# Default target
help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  build         Build WASM for bundlers (Vite, webpack)"
	@echo "  build-web     Build WASM for direct browser use"
	@echo "  build-node    Build WASM for Node.js"
	@echo "  test          Run Rust tests"
	@echo "  test-wasm     Run WASM tests in headless browser"
	@echo "  clean         Remove build artifacts"
	@echo "  link          Link package globally for local dev"
	@echo "  unlink        Unlink package globally"
	@echo "  publish       Publish to npm (requires npm login)"

# Build for bundlers (default)
build: build-bundler

build-bundler:
	wasm-pack build --target bundler --release

build-web:
	wasm-pack build --target web --release

build-node:
	wasm-pack build --target nodejs --release

# Run tests
test:
	cargo test

test-wasm:
	wasm-pack test --headless --chrome

# Clean build artifacts
clean:
	cargo clean
	rm -rf pkg/

# Link for local development
link: build
	cd pkg && pnpm link --global

unlink:
	cd pkg && pnpm unlink --global

# Publish to npm
publish: build
	cd pkg && npm publish --access public
