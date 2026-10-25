Summoner AI Rules
This project is a Rust DAW.

Priorities:

cargo check passes
cargo test passes
existing roadmap items work
bugs are reduced

Never:

rename files unnecessarily
move directories unnecessarily
rewrite architecture without justification
perform large cosmetic refactors

Prefer:

small localized fixes
compiler-driven changes
preserving existing public APIs
Always explain major changes before applying them.

{
  "name": "read_file",
  "arguments": {
    "filepath": "Cargo.toml"
  }
}

