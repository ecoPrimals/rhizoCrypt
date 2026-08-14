<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Contributing to rhizoCrypt

rhizoCrypt is part of the [ecoPrimals](https://github.com/ecoPrimals) sovereign
computing ecosystem. Contributions are welcome under the terms of the
[scyBorg Triple-Copyleft license](LICENSE).

## Prerequisites

- **Rust** — edition 2024, MSRV 1.87 (install via [rustup](https://rustup.rs))
- **cargo-deny** — `cargo install cargo-deny` (supply chain audit)
- **cargo-llvm-cov** — `cargo install cargo-llvm-cov` (coverage)

## Standards

All code must comply with the
[wateringHole STANDARDS_AND_EXPECTATIONS](https://github.com/ecoPrimals/wateringHole).
Key requirements:

| Rule | Enforcement |
|------|-------------|
| 100% Rust application code | `cargo-deny` bans C-sys crates |
| `#![forbid(unsafe_code)]` | Workspace lint policy |
| `clippy::pedantic` + `nursery` + `cargo` | Zero warnings (`-D warnings`) |
| `unwrap_used` / `expect_used` = `"deny"` | Production code only; tests use `#[expect(...)]` |
| No TODO/FIXME/HACK in committed code | CI enforced |
| Max 800 lines per file | Split into modules when approaching limit |
| `cargo fmt` | Must pass `--check` with no diff |
| `cargo doc` | Must generate with zero warnings |
| `cargo test --workspace --all-features` | Zero failures |
| Coverage gate | 90% lines (llvm-cov) |

## Capability-Based Architecture

rhizoCrypt discovers sibling capabilities at runtime. Production code must not:

- Import other primal crates directly
- Hardcode primal names in logic (comments should use capability-based language)
- Assume specific providers — use the `SigningProvider`, `PermanentStorageProvider`,
  and `PayloadStorageProvider` traits

## Development Workflow

```bash
# Format
cargo fmt --all

# Lint (must be zero warnings)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Test
cargo test --workspace --all-features

# Coverage (must be >= 90%)
cargo llvm-cov --workspace --all-features --summary-only

# Supply chain audit
cargo deny check

# Documentation
cargo doc --workspace --no-deps
```

## IPC Conventions

- Method names follow `domain.verb` semantic naming (`dag.session.create`)
- JSON-RPC 2.0 is the universal wire format; tarpc is the performance path
- G65 protocol negotiation on a single socket
- BTSP for family-scoped authentication
- UDS unconditional on Unix; TCP opt-in via `--port`

## License

By contributing you agree that your contributions will be licensed under:

- **Code**: AGPL-3.0-or-later
- **Game mechanics**: ORC
- **Documentation**: CC-BY-SA 4.0
