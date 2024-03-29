SHELL := /bin/bash
.PHONY: help clean lint format test doc build run bump 

help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

clean: ## Remove all build artifacts
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
	cargo build

all: clean lint format test doc build ## Build and run

bump: ## Bump version
	@echo "Current version: $$(cargo pkgid | grep -o '#.*' | cut -d# -f2)"
	@read -p "Enter new version: " new_version && \
	sed -i "s/version = \".*\"/version = \"$$new_version\"/" Cargo.toml && \
	echo "Updated to new version: $$(cargo pkgid | grep -o '#.*' | cut -d# -f2)"

build-windows: ## Build for Windows
	@rustup target add x86_64-pc-windows-gnu
	cargo build --target x86_64-pc-windows-gnu