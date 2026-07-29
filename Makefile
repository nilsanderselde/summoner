# Summoner DAW Makefile

.PHONY: help vibe vibe-py vibe-ps run build release test check clippy fmt fmt-fix generate presets clean

# Default target
.DEFAULT_GOAL := help

## help: Display available make targets
help:
	@echo "Summoner DAW - Available Make Targets:"
	@echo ""
	@echo "  Vibe Coding Autonomous Runners:"
	@echo "    make vibe        - Run vibe-coding autonomous loop (Bash: scripts/smart-vibe.sh)"
	@echo "    make vibe-py     - Run vibe-coding autonomous loop (Python stream: scripts/vibe_runner.py)"
	@echo "    make vibe-ps     - Run vibe-coding autonomous loop (PowerShell: scripts/smart-vibe.ps1)"
	@echo ""
	@echo "  Cargo & Rust Operations:"
	@echo "    make run         - Run main Summoner application (cargo run -p summon)"
	@echo "    make build       - Build debug workspace (cargo build --workspace)"
	@echo "    make release     - Build release workspace (cargo build --workspace --release)"
	@echo "    make test        - Run all unit & integration tests (cargo test --workspace)"
	@echo "    make check       - Quickly check workspace syntax & types (cargo check --workspace)"
	@echo "    make clippy      - Run clippy linter (cargo clippy --workspace --all-targets)"
	@echo "    make fmt         - Check code formatting (cargo fmt --all -- --check)"
	@echo "    make fmt-fix     - Automatically format code (cargo fmt --all)"
	@echo "    make clean       - Clean cargo target build directory (cargo clean)"
	@echo ""
	@echo "  Asset & Script Generators:"
	@echo "    make generate    - Run main audio/preset generator (python generate.py)"
	@echo "    make presets     - Generate drum presets (python generate_drum_presets.py)"

## vibe: Run the autonomous vibe coding bash script
vibe:
	bash ./scripts/smart-vibe.sh

## vibe-py: Run the streaming Python vibe coding script
vibe-py:
	python ./scripts/vibe_runner.py

## vibe-ps: Run the PowerShell vibe coding script
vibe-ps:
	powershell -ExecutionPolicy Bypass -File ./scripts/smart-vibe.ps1

## run: Run the main Summoner DAW binary
run:
	cargo run -p summon

## build: Build the workspace in debug mode
build:
	cargo build --workspace

## release: Build the workspace in release mode
release:
	cargo build --workspace --release

## test: Run all tests in the workspace
test:
	cargo test --workspace

## check: Check compilation without producing binaries
check:
	cargo check --workspace

## clippy: Run clippy linter across all workspace targets
clippy:
	cargo clippy --workspace --all-targets

## fmt: Check code formatting
fmt:
	cargo fmt --all -- --check

## fmt-fix: Auto-format code with rustfmt
fmt-fix:
	cargo fmt --all

## generate: Execute generate.py preset/audio script
generate:
	python generate.py

## presets: Execute generate_drum_presets.py script
presets:
	python generate_drum_presets.py

## clean: Remove cargo build target directory
clean:
	cargo clean
