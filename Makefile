SHELL := /bin/bash
WORKSPACE := $(shell pwd)
IMAGE := chip8-devcontainer
USER := $(shell id -u)
GROUP := $(shell id -g)

.PHONY: help clean lint format test doc build run bump build-windows 

help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

clean: ## Remove all build artifacts
	@rm -rf build
	cargo clean

lint: ## Lint code
	@rustup component add rustfmt 2> /dev/null
	cargo clippy 

format: ## Format code
	@rustup component add rustfmt 2> /dev/null
	cargo fmt

test: ## Run tests
	cargo test

doc: ## Generate documentation
	cargo doc --no-deps

bench: ## Run benchmarks
	cargo bench

build: ## Build
	cargo build --release
	@mkdir -p build/linux
	@cp target/release/chip8-cli build/linux/chip8
	@cp target/release/libSDL2* build/linux/

build-windows: ## Build for Windows
	@rustup target add x86_64-pc-windows-gnu
	cargo build --release --target x86_64-pc-windows-gnu
	@mkdir -p build/windows
	@cp target/x86_64-pc-windows-gnu/release/chip8-cli.exe build/windows/chip8.exe
	@cp target/x86_64-pc-windows-gnu/release/SDL2.dll build/windows

build-linux-with-docker: ## Build for Windows with Docker
	@docker build . -f .devcontainer/Dockerfile -t $(IMAGE) 
	@docker run --rm -v $(WORKSPACE):/workspace -w /workspace $(IMAGE) make build
	@docker run --rm -v $(WORKSPACE):/workspace -w /workspace $(IMAGE) chown -R $(USER):$(GROUP) build target

build-windows-with-docker: ## Build for Windows with Docker
	@docker build . -f .devcontainer/Dockerfile -t $(IMAGE) 
	@docker run --rm -v $(WORKSPACE):/workspace -w /workspace $(IMAGE) make build-windows
	@docker run --rm -v $(WORKSPACE):/workspace -w /workspace $(IMAGE) chown -R $(USER):$(GROUP) build target

all: clean lint format test doc build ## Build 

bump: ## Bump version
	@echo "Current version: $$(cargo pkgid | grep -o '#.*' | cut -d# -f2)"
	@read -p "Enter new version: " new_version && \
	sed -i "s/version = \".*\"/version = \"$$new_version\"/" Cargo.toml && \
	echo "Updated to new version: $$(cargo pkgid | grep -o '#.*' | cut -d# -f2)"
