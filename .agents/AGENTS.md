# Workspace Rules for Summoner

## Product North Star
Summoner is an instrument for experts and novices alike, geniuses and commonfolk alike: deeper key/harmony control than any other DAW, everything automatable, the ability to synthesize unheard-of sounds and manipulate audio, and enough on its own to produce IDM/EDM masterpieces. Third-party plugins (any plugin format) are a bonus, never a dependency. When a task is ambiguous, prefer the option that:
1. Keeps every parameter automatable and every note harmonically aware (scale/root/tuning are first-class).
2. Ships a real, audible implementation. No stubs, no `PassthroughNode` fallbacks for catalog entries; `cargo run -p summon -- capabilities --deny-simulated` must stay at 0 simulated.
3. Is deterministic and headless-first (offline render == realtime render).
4. Works on Windows, Linux and macOS.
5. Is discoverable and intuitive in the GUI.

## Definition of Done (single gate)
- Run `make verify` (fmt check, clippy `-D warnings` with `summon/gui`, full tests, capabilities gate). It mirrors CI. Never commit Rust changes unless it exits 0 (for quick iteration use `cargo test -p <crate>`; run `make verify` before committing).
- Windows note: `rg` is not installed; use PowerShell `Select-String` or the file tools. Run flaky-under-parallelism audio tests with `-- --test-threads=1` if they fail only in parallel, and say so in the report.

## Real-Time Audio Contract (inviolable)
- No heap allocation, locks, I/O, logging, or `format!` in `AudioNode::process` / `handle_event` or anything they call. Pre-allocate in `new()`.
- Every `NodeFactory::create_node_with_rate` arm is auto-checked by `crates/summoner_engine/tests/rt_safety.rs` under `AllocGuard`; a new arm that allocates fails the build.
- New DSP needs: a unit test that proves audible output, plus a zero-allocation guarantee.

## Architecture
- **State source of truth**: `summoner_project`. The GUI is a deterministic projection and holds no non-ephemeral semantic state; all mutations route through `summoner_project`.
- **Isolation**: build components (GUI widgets, DSP nodes) in isolation with unit tests. Wiring into `app.rs` / graph topology / `NodeFactory` is an integration task; do it deliberately and only when tasked.
- **Architecture changes are allowed** when they unblock a roadmap item. Write a short decision note in `local/` first, then proceed. Do not do drive-by cosmetic refactors or needless file moves.
- **File size**: do not grow a file past ~1,500 lines; propose a split instead (large files cost repeated expensive re-reads). Known offenders are tracked in the roadmap.

## Roadmap Management
- The latest dated `local/ROADMAP_YYYYMMDD_HHMMSS.md` is authoritative; older ones are history.
- When the current roadmap is 100% complete, create the next one before finishing.
- `local/` is its own private Git repo (remote `summoner-prompts`) and is gitignored by the root repo. Never `git add -f` it from the root. After creating or editing anything in `local/`: `cd local && git add ... && git commit -m "..." && git push`.

## Git Workflow (commit & push after every verified step)
- Commit with a clear, detailed message right after each verified step/widget/sub-feature. Never leave uncommitted or unpushed work at the end of a turn.
- Work on the `dev` branch and push it (`git push origin dev`); CI runs on `dev`. Promote to `master` with `gh pr create --base master --head dev` then `gh pr merge --auto --rebase` so the 7 required checks gate `master` (linear history required). Also push `local/`.
- Never force-push `master`. Never disable branch protection to get a push through.

## Quota Discipline
- Prefer the cheapest model that can do the job; delegate wide read-only research to subagents.
- Do not re-read large files you already have; use ranged reads (<= 100 lines) and edit from known text.
- If a task needs a design decision the roadmap does not answer, or would take a large number of extra reads, write the question into `local/` and stop instead of improvising.
- Persist requirements to `local/` before starting big implementation work so nothing is lost if quota runs out.

## Environment & Makefile
- Use `python` for Python scripts. `python`, `python3`, `py`, `make` are authorized in workspace scripts (e.g. `scripts/vibe_runner.py`, `scripts/smart-vibe.sh`, `generate.py`).
- `make verify` (gate), `make test`, `make check`, `make build`, `make run`, `make clippy`, `make fmt`, `make fmt-fix`; `make vibe` runs the streaming vibe runner (`scripts/vibe_runner.py`).

## Tool Usage Directives
- NEVER pass `ArtifactMetadata` when writing/editing workspace files (`crates/...`, `src/...`, `scripts/...`, `local/...`, `.agents/...`); it is only for artifacts in the brain directory and causes fatal schema errors on repo files.

## General Project Rules
- **Prohibited Files**: Do NOT ever create, add, or suggest adding a `CODE_OF_CONDUCT.md`.
- Keep `AGENTS.md` short and non-contradictory; if a rule here conflicts with another doc, this file wins.
