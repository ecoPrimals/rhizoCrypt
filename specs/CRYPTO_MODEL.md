# rhizoCrypt — Canonical Cryptographic Model

**Version**: 1.0  
**Status**: Authoritative  
**Effective**: April 2026  
**Origin**: primalSpring audit item #3 (post-Phase 43)

---

## Decision

**rhizoCrypt delegates all asymmetric cryptography to BearDog via IPC.**

rhizoCrypt maintains Blake3 content-addressing for internal data integrity
but does not implement, store, or manage signing keys. All Ed25519 signing,
verification, and DID operations flow through a capability-discovered
`crypto.*` provider (canonically BearDog).

This decision follows the wateringHole `PRIMAL_RESPONSIBILITY_MATRIX.md`:
BearDog **OWNS** the crypto concern; the provenance trio is **N/A** on that
row. "Self-sovereign crypto" in the ecosystem means BearDog is the single
auditable crypto surface — not that every primal reimplements signing.

---

## Crypto Boundary

### Self (rhizoCrypt owns)

| Operation | Primitive | Purpose |
|-----------|-----------|---------|
| Vertex ID computation | Blake3 | Content-address vertices from CBOR-canonical body |
| Merkle tree construction | Blake3 `hash_pair` | DAG integrity — Merkle root over session vertices |
| Dehydration summary hash | Blake3 | Bind `(session_id, merkle_root, resolved_at)` for attestation input |
| Payload reference | Blake3 | Content-address large payloads |
| BTSP handshake | X25519 + HKDF-SHA256 + HMAC-SHA256 | Family-scoped transport authentication (Phase 2) |

These are **data integrity** operations. They produce hashes that downstream
consumers (including BearDog) can independently verify. No long-term keys
are held, no signatures are produced.

### Delegated (BearDog via `crypto.*` IPC)

| Operation | BearDog Method | When |
|-----------|----------------|------|
| Sign dehydration summary hash | `crypto.sign_contract` | During attestation collection (pre-commit) |
| Verify attestation signatures | `crypto.verify_contract` | When validating witness attestations |
| DID-based identity verification | `crypto.verify_ed25519` | Validating agent DIDs in session vertices |
| Ionic bond proposal/acceptance | `crypto.ionic_bond.*` | Cross-NUCLEUS trust establishment |
| General Ed25519 signing | `crypto.sign_ed25519` | Any operation requiring a signature |

### Not in Scope

| Operation | Why |
|-----------|-----|
| Key generation | BearDog derives keys from primal identity seed |
| Key storage | BearDog manages HSM/software keystore |
| TLS termination | BearDog or NestGate at network edge |
| Certificate management | BearDog's domain |

---

## Discovery Pattern

rhizoCrypt discovers the signing provider at runtime — never at compile time:

```
1. registry.find_by_capability(Capability::Signing)
2. → returns endpoint (UDS, TCP, or HTTP)
3. → SigningClient::connect(endpoint)
4. → client.sign(summary_hash, attester_did)
```

The `SigningClient` adapter is protocol-agnostic. BearDog is the canonical
implementation, but any service exposing `crypto.sign_*` / `crypto.verify_*`
JSON-RPC methods satisfies the contract.

---

## Dehydration Attestation Flow

```
rhizoCrypt                    BearDog (discovered)
    │                              │
    ├─ compute_merkle_root()       │  ← Blake3 (self)
    ├─ build_summary()             │
    ├─ compute_hash()              │  ← Blake3 (self)
    │                              │
    ├─ crypto.sign_contract ──────►│  ← Ed25519 (delegated)
    │◄── { signature, pubkey } ────┤
    │                              │
    ├─ attach attestation          │
    ├─ commit to loamSpine ──────► │  (separate capability)
    │                              │
```

When `DehydrationConfig::required_attestations` is non-empty, rhizoCrypt
calls the discovered signing provider for each required attester. The
provider signs `summary_hash.as_bytes()` with the attester's identity key
and returns `(signature, public_key)`.

For verification, rhizoCrypt calls `crypto.verify_contract` with the
stored `(terms_hash, signature, public_key)` triplet.

---

## BTSP Transport Crypto (Phases)

| Phase | Status | What |
|-------|--------|------|
| Phase 1 | Complete | Family-scoped socket naming (`rhizocrypt-{family_id}.sock`) |
| Phase 2 | Complete | X25519 handshake + HKDF session keys + HMAC family proof |
| Phase 3 | Planned | Per-frame ChaCha20-Poly1305 AEAD using derived session keys |

BTSP crypto (X25519, HKDF, HMAC) lives in `rhizo-crypt-rpc` because it is
**transport-layer** authentication, not application-level signing. The
primitives are used only during connection establishment — no long-term
keys are generated or stored.

Phase 3 will encrypt JSON-RPC frames after the handshake using the session
keys already derived in Phase 2. The cipher infrastructure
(`BtspCipher::Chacha20Poly1305`) is defined but not wired to the post-
handshake read/write path yet.

---

## What This Means for Evolution

1. **No signing keys in rhizoCrypt** — ever. Content hashes are sufficient
   for DAG integrity; provenance signatures belong to BearDog.

2. **Attestation requires a live signing provider** — dehydration with
   `required_attestations` fails gracefully when no `crypto.*` provider is
   discoverable (standalone mode).

3. **Ionic bonds are BearDog's concern** — rhizoCrypt may *participate* in
   ionic bond verification (checking that a bond exists and is sealed) but
   does not *create* bonds. Bond creation flows through `crypto.ionic_bond.*`.

4. **Wire format alignment** — `SigningClient` and `BearDogHttpClient`
   converge on BearDog's semantic method names (`crypto.sign_contract`,
   `crypto.verify_contract`) rather than legacy REST paths.

5. **BTSP Phase 3** is a natural next step that uses only the session keys
   already derived — no new crypto primitives needed.

---

## Known Evolution Gaps

### Wire Format Alignment (post-Phase 43)

`SigningClient` currently uses abstract method names (`sign`, `verify`,
`attest`) and its own DTO shapes (`SignRequest { data, signer }`,
`AttestRequest { attester, summary }`). BearDog's actual JSON-RPC methods
use semantic names and different field shapes:

| rhizoCrypt adapter | BearDog UDS method | Status |
|--------------------|-------------------|--------|
| `sign` | `crypto.sign_ed25519` | Field mismatch: `data`/`signer` vs `message`/`key_id` |
| `verify` | `crypto.verify_ed25519` | Field mismatch: `data`/`signature` vs `message`/`signature` |
| `attest` | `crypto.sign_contract` | Conceptual match: summary→terms, attester→signer |
| `verify_did` | (no equivalent yet) | BearDog DID types present but not wired |

**Evolution path**: Align `SigningClient` method names to `crypto.*` semantic
namespace and bridge DTOs through an adapter-level translation layer, or
evolve both sides to a shared wire schema.

---

## References

- wateringHole `PRIMAL_RESPONSIBILITY_MATRIX.md` — BearDog OWNS crypto
- wateringHole `PRIMAL_IPC_PROTOCOL.md` v3.1 — `crypto.sign` discovery pattern
- wateringHole `CAPABILITY_WIRE_STANDARD.md` — identity key + ionic bond signing
- wateringHole `SPRING_COORDINATION_AND_VALIDATION.md` — provenance trio routing
- BearDog `ionic_bond.rs` — contract and bond wire types
- BearDog `primal_signing.rs` — Ed25519 primal identity key derivation
