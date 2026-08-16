# Workspace Rules for Summoner

## Environment & Python Configuration
- Use `python` to execute Python scripts.
- Python command prefixes (`python`, `python3`, `py`, `make`) are authorized for execution within workspace scripts (e.g. `scripts/vibe_runner.py`, `scripts/smart-vibe.sh`, `generate.py`).

## Roadmap Management Rules
- **Authoritative Roadmap**: Always discover and treat the latest dated `local/ROADMAP_YYYYMMDD_HHMMSS.md` file as authoritative.
- **Historical Context**: Read older roadmap files in `local/` if needed for historical context or previous analysis.
- **New Roadmap Generation**: Once all tasks in the current roadmap are 100% complete, create a new roadmap file `local/ROADMAP_YYYYMMDD_HHMMSS.md` containing the next phase of milestones before finishing.

## Makefile Targets & Workflows
- `make vibe` runs the autonomous vibe coding bash runner script (`scripts/smart-vibe.sh`).
- `make vibe-py` runs the streaming Python runner (`scripts/vibe_runner.py`).
- `make test`, `make check`, `make build`, `make run`, `make clippy`, `make fmt` provide workspace shortcuts.

## Git Commit Requirement
- Commit changed files intermittently with a clear, detailed Git commit message immediately after implementing and verifying each step, widget, or sub-feature with tests. Do NOT accumulate uncommitted changes across multiple steps or leave uncommitted edits at the end of turns so work is never lost when quota limits are reached.

## Subagent Codebase & Verification Directives
- **Mandatory Self-Correction Loop**: Every agent *must* run `cargo check`, `cargo clippy`, and `cargo test` after modifying Rust code. They must self-correct any warnings or errors *before* finishing their turn.
- **State Source of Truth**: `summoner_project` is the absolute source of truth. The GUI is strictly a deterministic projection and cannot hold non-ephemeral semantic state. All state mutations must be routed to `summoner_project`.
- **Extreme Isolation (Codebase Seams)**: Subagents tasked with building components (GUI widgets, DSP nodes) MUST build them in isolation with unit tests. They are forbidden from wiring them into monolithic states (`app.rs` / graph topologies) unless specifically tasked as an integration agent.

## General Project Rules
- **Prohibited Files**: Do NOT ever create, add, or suggest adding a `CODE_OF_CONDUCT.md` file to this repository.
