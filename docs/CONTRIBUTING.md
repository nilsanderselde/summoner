# Contributing to Summoner DAW

Thank you for your interest in contributing to **Summoner DAW**! This guide details development practices, real-time safety guardrails, testing workflows, and code submission standards.

---

## 1. Code Guidelines & Real-Time Safety Rules

Summoner DAW enforces strict real-time audio constraints to prevent xruns, audio glitches, and non-deterministic behavior.

### Critical Guardrails

1. **Zero Heap Allocation in DSP Loop:**
   Never perform heap allocations inside `process_block` (`process()`). Pre-allocate all buffers, vectors, node structures, and temporary scratch space during node initialization.
2. **Lock-Free Execution:**
   Do not use standard mutexes (`std::sync::Mutex`), read-write locks (`RwLock`), or blocking channels inside audio rendering code. Use lock-free atomics (`AtomicF32`, `AtomicU32`) or lock-free SPSC queues (`AtomicRingBuffer`).
3. **Bit-Exact Determinism:**
   Audio rendering must produce identical WAV sample output across platforms given the same inputs and seeds. Avoid non-deterministic OS dependencies inside DSP routines.
4. **Preserve Documentation:**
   Maintain existing doc comments (`///`) and module level docs (`//!`). Run `make test` to verify that all doctests pass.

---

## 2. Development & Build Commands

We provide shortcuts via `Makefile`:

```bash
# Check compilation and clippy lints across all workspace crates
make check

# Run unit tests and doctests
make test

# Format codebase according to Rust guidelines
make fmt

# Build release binaries
make build
```

---

## 3. Testing & Verification Requirements

Before submitting code, ensure that all test suites pass:

1. **Unit & Integration Tests:** Run `cargo test --workspace`.
2. **Doctests:** Verify that doctests in module docs execute cleanly.
3. **Preset Schema Validation:** Ensure presets parse and validate against `local/preset.schema.json`.
4. **Fuzzing Targets:** If modifying DSP algorithms or audio parsers (WAV/FLAC), test against LibFuzzer targets in `fuzz/fuzz_targets/`:
   ```bash
   cargo fuzz run fuzz_filter_ladder
   cargo fuzz run fuzz_filter_svf
   ```

---

## 4. Git Commit Message Conventions

Commit messages must follow the Conventional Commits specification:

- `feat(crate): description` for new features
- `fix(crate): description` for bug fixes
- `docs(crate): description` for documentation updates
- `refactor(crate): description` for code cleanup without functional changes

Example:
```git
feat(dsp): implement PolyBLEP anti-aliased OscSaw primitive

- Add PolyBLEP residual calculation for step discontinuities
- Add V/Oct pitch input and hard sync phase reset
- Add unit tests verifying aliasing attenuation
```
