+++
title = "rhizoCrypt Validation Summary"
description = "Ephemeral DAG engine — 1,670 tests, 36 methods, pure Rust, content-addressed working memory for the ecoPrimals ecosystem"
date = 2026-06-03

[taxonomies]
primals = ["rhizocrypt"]
springs = []
+++

## Status

- **1,670 tests** passing (unit + integration + property + doc, `--all-features`)
- **181 `.rs` files**, ~54,825 lines
- **36 registered methods** across 6 domains (31 stable, 5 evolving)
- **93.88% line coverage** (CI gate: 90%)
- **Zero `unsafe` blocks** — `unsafe_code = "deny"` workspace-wide
- **Zero C dependencies** — ecoBin compliant, `cargo-deny` enforced
- **Edition 2024**, Rust 1.87 MSRV
- **BTSP Phase 3** — ChaCha20-Poly1305 encrypted channels on UDS

## Capability Domains

| Domain | Methods | Stability | Purpose |
|--------|---------|-----------|---------|
| `dag.session.*` | 4 | Stable | Session lifecycle (create, get, list, discard) |
| `dag.event.*` | 2 | Stable | Vertex append (single + batch) |
| `dag.vertex.*` | 3 | Stable | Vertex retrieval and query |
| `dag.merkle.*` | 3 | Stable | Merkle root, proof, verify |
| `dag.slice.*` | 4 | Stable | Immutable snapshot checkout/resolve |
| `dag.dehydration.*` | 2 | Stable | Commit to permanent storage |
| `dag.partial_dehydrate` | 1 | Evolving | Partial Merkle root without closing session |
| `dag.branch/diff/merge/federate` | 4 | Evolving | DAG evolution — branch, diff, merge, cross-gate federation (Wave 60) |
| `health.*` | 4 | Stable | Liveness, readiness, check, metrics |
| `auth.*` | 3 | Stable | Method gate introspection (JH-0/JH-1) |
| `identity/capabilities/tools` | 6 | Stable | Discovery, MCP tool exposure |

## Wire-Name Aliases

`provenance.*` methods (e.g. `provenance.session.create`) are normalized
to `dag.*` at dispatch time. Both naming conventions are valid on the wire
(GAP-36 resolution, S68).

## Composition Role

rhizoCrypt is the **ephemeral working memory** of the Provenance Trio:

```
rhizoCrypt (DAG) → loamSpine (ledger) → sweetGrass (attribution)
```

Downstream consumers use `dag.session.create` → `dag.event.append` →
`dag.partial_dehydrate` / `dag.dehydration.trigger` to produce
cryptographically verified Merkle roots that feed loamSpine commits
and sweetGrass braids.

## Downstream Pairing

| Partner | Role | Key Methods |
|---------|------|-------------|
| wetSpring | DAG checkpointing for LTEE pipelines | `dag.partial_dehydrate`, `dag.event.append` |
| lithoSpore | Provenance DAG verification substrate | `dag.merkle.root`, `dag.merkle.proof` |
| projectFOUNDATION | Thread lineage evidence | `dag.session.get` (summary) |
| healthSpring | Nest atomic clinical pipeline | `provenance.*` aliases |

## Transport

- **UDS**: Unconditional on Unix (`$XDG_RUNTIME_DIR/biomeos/rhizocrypt.sock`)
- **TCP**: Opt-in (`--port` / `RHIZOCRYPT_PORT`)
- **BTSP**: Mandatory when `FAMILY_ID` is set (Phase 1/2/3)
- **Stale socket cleanup**: `unlink()` before `bind()` + shutdown cleanup
- **Neural API**: `primal.announce` on startup (Wave 43) — registers with biomeOS for routing (background, non-blocking since Wave 49)

## Deep Debt Posture

| Category | Status |
|----------|--------|
| `unsafe` blocks | Zero (`deny`) |
| `async-trait` | Zero |
| `Arc<Mutex>` | Zero |
| `Box<dyn Error>` (production) | Zero |
| `unwrap`/`expect` (production) | Zero (`deny`) |
| TODO/FIXME/HACK | Zero |
| `&Vec<`/`&String` params | Zero |
| Production mocks | Zero (all `cfg`-gated) |
| C dependencies | Zero (ecoBin) |

## See Also

- [Capability Registry](../capability_registry.toml) — 36 methods with stability tiers
- [API Specification](../specs/API_SPECIFICATION.md) — tarpc + JSON-RPC wire format
- [CHANGELOG](../CHANGELOG.md) — full evolution history
