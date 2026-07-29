# Workspace Rules for Summoner

## Environment & Python Configuration
- Use `python` to execute Python scripts (located in system PATH at `C:\Users\Nils\AppData\Local\Programs\Python\Python313\python.exe`).
- Python command prefixes (`python`, `python3`, `py`, `make`) are authorized for execution within workspace scripts (e.g. `scripts/vibe_runner.py`, `scripts/smart-vibe.sh`, `generate.py`).

## Makefile Targets & Workflows
- `make vibe` runs the autonomous vibe coding bash runner script (`scripts/smart-vibe.sh`).
- `make vibe-py` runs the streaming Python runner (`scripts/vibe_runner.py`).
- `make vibe-ps` runs the PowerShell runner (`scripts/smart-vibe.ps1`).
- `make test`, `make check`, `make build`, `make run`, `make clippy`, `make fmt` provide workspace shortcuts.

## Git Commit Requirement
- After implementing each step or feature and running tests to verify, commit all changed files with a clear, detailed Git commit message before finishing or beginning the next step.
