# Summoner AI Rules

The authoritative agent rules live in [`.agents/AGENTS.md`](.agents/AGENTS.md). This file is a short summary kept for tools that look for `AI_RULES.md`.

- This project is a Rust DAW. `make verify` is the single definition of done.
- Respect the real-time audio contract (no allocation/locks/I/O in audio callbacks).
- Prefer small, compiler-driven, API-preserving changes. Architecture changes are allowed when they unblock a roadmap item, after a short decision note in `local/`.
- Avoid unnecessary renames, directory moves, and cosmetic refactors.
- Explain major changes before applying them.
