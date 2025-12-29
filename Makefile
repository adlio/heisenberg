# Heisenberg Build System
# Requires: cargo-nextest, cargo-llvm-cov (auto-installed if missing)

.DEFAULT_GOAL := help

.PHONY: help test coverage coverage-html build build-release clean fmt fmt-check lint check all ci ensure-tools bench npm-audit-fix doc doc-check examples examples-frontend

# Tool installation helpers
CARGO_NEXTEST := $(shell command -v cargo-nextest 2>/dev/null)
CARGO_LLVM_COV := $(shell command -v cargo-llvm-cov 2>/dev/null)
NODE := $(shell command -v node 2>/dev/null)

ensure-tools: ## Install required cargo tools if missing
ifndef CARGO_NEXTEST
	@echo "Installing cargo-nextest..."
	@cargo install cargo-nextest --locked
endif
ifndef CARGO_LLVM_COV
	@echo "Installing cargo-llvm-cov..."
	@cargo install cargo-llvm-cov --locked
endif
ifndef NODE
	@echo "Warning: Node.js not found. Required for building examples."
endif

help: ## Show available targets
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

all: ensure-tools fmt lint build test ## Format, lint, build, and test

ci: ensure-tools fmt-check lint build test doc-check examples ## Full CI checks (superset of GitHub CI)


test: ensure-tools ## Run tests with nextest
	cargo nextest run --workspace --all-features

coverage: ensure-tools ## Show coverage summary in console
	cargo llvm-cov nextest --workspace --all-features

coverage-html: ensure-tools ## Generate HTML coverage report and open
	cargo llvm-cov nextest --workspace --all-features --html
	open target/llvm-cov/html/index.html

build: ## Build debug
	cargo build --workspace --all-targets --all-features

build-release: ## Build release
	cargo build --workspace --all-targets --all-features --release

check: ## Run cargo check
	cargo check --workspace --all-targets --all-features

fmt: ## Format code
	cargo fmt --all

fmt-check: ## Check formatting
	cargo fmt --all -- --check

lint: ## Run clippy
	cargo clippy --workspace --all-targets --all-features -- -D warnings

clean: ## Clean build artifacts
	cargo clean
	find examples -name "node_modules" -type d -exec rm -rf {} + 2>/dev/null || true
	find examples -name "dist" -type d -exec rm -rf {} + 2>/dev/null || true
	find examples -name "build" -type d -exec rm -rf {} + 2>/dev/null || true
	find examples -name "custom-dist" -type d -exec rm -rf {} + 2>/dev/null || true
	find examples -name "custom-build" -type d -exec rm -rf {} + 2>/dev/null || true
	find examples -name ".frontend-built" -exec rm -f {} + 2>/dev/null || true
	find examples -name "target" -type d -exec rm -rf {} + 2>/dev/null || true

doc: ## Generate docs and open
	cargo doc --workspace --no-deps --open

doc-check: ## Check docs build without warnings
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

bench: ## Run performance benchmarks
	cargo bench --bench performance

npm-audit-fix: ## Run npm audit fix in all example packages
	@for pkg in $$(find examples -name "package.json" -not -path "*/node_modules/*"); do \
		dir=$$(dirname $$pkg); \
		echo "=== Auditing $$dir ==="; \
		(cd $$dir && npm audit fix) || true; \
	done

FRONTEND_SRC_PATTERNS := -name "*.json" -o -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.jsx" -o -name "*.svelte" -o -name "*.vue" -o -name "*.css" -o -name "*.html"

examples-frontend: ## Build all example frontend packages
	@for pkg in $$(find examples -name "package.json" -not -path "*/node_modules/*"); do \
		dir=$$(dirname $$pkg); \
		if [ -f "$$dir/.frontend-built" ] && [ -z "$$(find $$dir -not -path "*/node_modules/*" \( $(FRONTEND_SRC_PATTERNS) \) -newer $$dir/.frontend-built 2>/dev/null)" ]; then \
			echo "[examples] Skipping $$dir (already built)"; \
		else \
			echo ""; \
			echo "========================================"; \
			echo "[examples] Building frontend: $$dir"; \
			echo "========================================"; \
			(cd $$dir && npm ci && npm run build && touch .frontend-built); \
		fi \
	done

examples: examples-frontend ## Build all examples (frontend + Rust)
	@for dir in examples/*/; do \
		echo ""; \
		echo "========================================"; \
		echo "[examples] Building Rust: $$dir"; \
		echo "========================================"; \
		(cd $$dir && cargo build); \
	done
