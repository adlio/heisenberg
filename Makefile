.PHONY: build test coverage lint format check-all clean help npm-audit-fix

# Default target
help:
	@echo "Available targets:"
	@echo "  build     - Build the project"
	@echo "  test      - Run tests with nextest"
	@echo "  coverage  - Generate test coverage report"
	@echo "  lint      - Run clippy linter"
	@echo "  format    - Format code with rustfmt"
	@echo "  check-all - Run all quality checks"
	@echo "  bench     - Run performance benchmarks"
	@echo "  clean     - Clean build artifacts"
	@echo "  npm-audit-fix - Run npm audit fix in all example packages"

# Build the project
build:
	cargo build --all-features

# Run tests with nextest
test:
	cargo nextest run --all-features

# Generate coverage report
coverage:
	cargo llvm-cov nextest --all-features --html
	@echo "\n=== Coverage Summary ==="
	cargo llvm-cov nextest --all-features --summary-only

# Run clippy linter
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Format code
format:
	cargo fmt

# Check formatting without modifying files
format-check:
	cargo fmt --check

# Run all quality checks
check-all: format-check lint test

# Run benchmarks
bench:
	@echo "Running performance benchmarks..."
	cargo bench --bench performance

# Clean build artifacts
clean:
	cargo clean

# Run npm audit fix in all example packages
npm-audit-fix:
	@for pkg in $$(find examples -name "package.json" -not -path "*/node_modules/*"); do \
		dir=$$(dirname $$pkg); \
		echo "=== Auditing $$dir ==="; \
		(cd $$dir && npm audit fix) || true; \
	done
