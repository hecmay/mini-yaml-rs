.PHONY: build build-web build-bundler build-rust clean test publish link help

# Default target
help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  build-rust    Build Rust library (for Tauri/backend use)"
	@echo "  build         Build WASM for bundlers (Vite, webpack)"
	@echo "  build-web     Build WASM for direct browser use"
	@echo "  build-node    Build WASM for Node.js"
	@echo "  test          Run Rust tests"
	@echo "  test-wasm     Run WASM tests in headless browser"
	@echo "  clean         Remove build artifacts"
	@echo "  link          Link package globally for local dev"
	@echo "  unlink        Unlink package globally"
	@echo "  publish       Publish to npm (requires npm login)"

# Build Rust library (no WASM)
build-rust:
	cargo build --release

# Build for bundlers (default WASM target)
build: build-web

build-bundler:
	wasm-pack build --target bundler --release --features wasm

build-web:
	wasm-pack build --target web --release --features wasm

build-node:
	wasm-pack build --target nodejs --release --features wasm

# Run tests
test:
	cargo test

test-wasm:
	wasm-pack test --headless --chrome --features wasm

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
