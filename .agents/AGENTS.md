# Workspace Rules for Summoner

## Environment & Python Configuration
- Use `python` to execute Python scripts (located in system PATH at `C:\Users\Nils\AppData\Local\Programs\Python\Python313\python.exe`).
- Python command prefixes (`python`, `python3`, `py`, `make`) are authorized for execution within workspace scripts (e.g. `scripts/vibe_runner.py`, `scripts/smart-vibe.sh`, `generate.py`).

## Roadmap Management Rules
- **Authoritative Roadmap**: Always discover and treat the latest dated `local/ROADMAP_YYYYMMDD.md` file as authoritative.
- **Historical Context**: Read older roadmap files in `local/` if needed for historical context or previous analysis.
- **New Roadmap Generation**: Once all tasks in the current roadmap are 100% complete, create a new roadmap file `local/ROADMAP_YYYYMMDD.md` containing the next phase of milestones before finishing.

## Makefile Targets & Workflows
- `make vibe` runs the autonomous vibe coding bash runner script (`scripts/smart-vibe.sh`).
- `make vibe-py` runs the streaming Python runner (`scripts/vibe_runner.py`).
- `make test`, `make check`, `make build`, `make run`, `make clippy`, `make fmt` provide workspace shortcuts.

## Git Commit Requirement
- After implementing each step or feature and running tests to verify, commit all changed files with a clear, detailed Git commit message before finishing or beginning the next step.
