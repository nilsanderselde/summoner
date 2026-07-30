# Security Policy

## Supported Versions

Summoner DAW security updates are applied to the `master` branch and the latest minor release version.

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We take the security of Summoner DAW seriously. If you believe you have found a security vulnerability in any aspect of Summoner DAW (including audio engine memory safety, DSP node processing, CLI tools, GUI components, or plugin isolation), please report it to us as follows:

1. **Email**: Send details of the vulnerability to `TBD` (or contact core maintainers directly via private GitHub security advisories).
2. **Details to Include**:
   - Description of the vulnerability and potential impact.
   - Steps to reproduce or a minimal proof-of-concept (PoC) project/file.
   - Any suggested mitigations or fixes.
3. **Response Time**: You will receive an initial response acknowledging your report within 48 hours.
4. **Coordinated Disclosure**: We request that you give maintainers reasonable time (up to 90 days) to address the issue before public disclosure.

## Security Architecture & Design Principles

Summoner DAW enforces strict real-time and system-level security constraints:

1. **Real-time Audio Callback Safety**:
   - Zero heap allocations (`AllocGuard` verified) in real-time processing blocks to eliminate audio dropouts and latency spikes.
   - Real-time threads never perform blocking I/O, lock contention, or uncontrolled pointer operations.

2. **Memory Safety & Unsafe Code Policy**:
   - Pure Rust codebase across audio nodes, DSP filters, and GUI components.
   - Strict `#![forbid(unsafe_code)]` or documented safety invariants where `unsafe` is strictly required.

3. **Plugin Sandboxing**:
   - Out-of-process isolation for third-party CLAP/VST3 plugins to prevent host process crashes or unauthorized system access.

4. **Dependency Auditing**:
   - Continuous dependency vulnerability scanning via `cargo-audit` and license enforcement via `cargo-deny`.
