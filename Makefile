# Summoner DAW Makefile

.PHONY: help vibe vibe-py run build release test check clippy fmt fmt-fix generate presets clean

# Default target
.DEFAULT_GOAL := help

## help: Display available make targets
help:
	@echo "Summoner DAW - Available Make Targets:"
	@echo ""
	@echo "  Vibe Coding Autonomous Runners:"
	@echo "    make vibe                  - Run vibe-coding autonomous loop (Python stream: scripts/vibe_runner.py)"
	@echo "    make vibe-finisher         - Run vibe-finisher autonomous loop (Python stream: scripts/vibe_finisher.py)"
	@echo "    make vibe-designer         - Run vibe-designer autonomous loop (Python stream: scripts/vibe_designer.py)"
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
	@echo ""
	@echo "  Deprecated/Alias Targets (Will be removed):"
	@echo "    make vibe-py               - Run vibe-coding (Alias for make vibe)"
	@echo "    make vibe-finisher-py      - Run vibe-finisher (Alias for make vibe-finisher)"
	@echo "    make vibe-designer-py      - Run vibe-designer (Alias for make vibe-designer)"

## vibe: Run the streaming Python vibe coding script directly
vibe: vibe-py

vibe-py:
	python ./scripts/vibe_runner.py

## vibe-finisher: Run the streaming Python vibe finisher script directly
vibe-finisher: vibe-finisher-py

vibe-finisher-py:
	python ./scripts/vibe_finisher.py

## vibe-designer: Run the streaming Python vibe GUI designer script directly
vibe-designer: vibe-designer-py

vibe-designer-py:
	python ./scripts/vibe_designer.py



## run: Run the main Summoner DAW binary
run:
	cargo run -p summon

## build: Build the workspace in debug mode
build:
	cargo build --workspace

## release: Build the workspace in release mode
release:
	cargo build --workspace --release

## build-gui: Build workspace with GUI feature enabled
build-gui:
	cargo build --workspace --features gui

## build-gui-release: Build release workspace with GUI feature enabled
build-gui-release:
	cargo build --workspace --features gui --release

## build-gui-windows: Cross-compile/Build GUI release binary for Windows
build-gui-windows:
	cargo build --workspace --features gui --release --target x86_64-pc-windows-msvc

## build-gui-linux: Cross-compile/Build GUI release binary for Linux (Debian/Arch)
build-gui-linux:
	cargo build --workspace --features gui --release --target x86_64-unknown-linux-gnu

## build-gui-macos: Cross-compile/Build GUI release binary for macOS
build-gui-macos:
	cargo build --workspace --features gui --release --target x86_64-apple-darwin

## test: Run all tests in the workspace
test:
	cargo test --workspace --features gui

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
