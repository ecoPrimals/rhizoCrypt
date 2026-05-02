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
| **Sign vertex on append** | `crypto.sign_ed25519` | Every `dag.event.append` when a signing provider is available and the vertex carries an `agent` DID |
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

## Vertex Signing Flow (Session 52)

```
dag.event.append (JSON-RPC)
    │
    ├─ VertexBuilder::build()          ← signature: None
    │
    ├─ sign_vertex_if_available()
    │     ├─ vertex.agent present?     ← skip if None
    │     ├─ primal.signing_client()   ← lazy discovery, cached
    │     │
    │     ├─ to_canonical_bytes()      ← CBOR (excludes signature)
    │     ├─ crypto.sign_ed25519 ─────►│ signing provider (discovered)
    │     │◄── { signature } ─────────┤
    │     └─ vertex.signature = sig    ← attached before append
    │
    ├─ vertex.id()                     ← Blake3 of canonical bytes
    ├─ dag_store.put_vertex()          ← stored with signature
    │
```

The signing flow is wired at the RPC service layer (`service.rs`), keeping
the core `append_vertex` method fast and pure (no IPC). The signing client
is lazily resolved via `RhizoCrypt::signing_client()` (backed by
`tokio::sync::OnceCell`) on first use, so standalone mode pays zero cost.

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
| Phase 3 | Complete | `btsp.negotiate` → ChaCha20-Poly1305 AEAD encrypted channel (Session 59) |

BTSP crypto (X25519, HKDF, HMAC, ChaCha20-Poly1305) lives in `rhizo-crypt-rpc`
because it is **transport-layer** authentication and encryption, not
application-level signing. Phase 2 primitives handle connection establishment;
Phase 3 encrypts all subsequent JSON-RPC traffic using HKDF-derived session
keys (`btsp-session-v1-c2s` / `btsp-session-v1-s2c`). No long-term keys are
generated or stored.

---

## What This Means for Evolution

1. **No signing keys in rhizoCrypt** — ever. Content hashes provide
   DAG integrity; cryptographic signatures for verifiability are delegated
   to the signing provider (canonically BearDog). The signing client is
   lazily resolved via capability discovery and cached for the primal's
   lifetime.

2. **Vertex signing on append** — when a signing provider is available
   and the vertex carries an `agent` DID, `dag.event.append` delegates
   to `crypto.sign_ed25519` before storage. This makes the DAG
   independently verifiable by any party holding the agent's public key.
   Vertices without an agent or without a provider remain unsigned
   (graceful degradation for standalone mode).

3. **Attestation requires a live signing provider** — dehydration with
   `required_attestations` fails gracefully when no `crypto.*` provider is
   discoverable (standalone mode).

4. **Ionic bonds are BearDog's concern** — rhizoCrypt may *participate* in
   ionic bond verification (checking that a bond exists and is sealed) but
   does not *create* bonds. Bond creation flows through `crypto.ionic_bond.*`.

5. **Wire format alignment** — `SigningClient` and `BearDogHttpClient`
   converge on BearDog's semantic method names (`crypto.sign_contract`,
   `crypto.verify_contract`) rather than legacy REST paths.

6. **BTSP Phase 3** (Session 59) encrypts all post-handshake traffic using
   ChaCha20-Poly1305 AEAD with HKDF-derived session keys — no new long-term
   crypto primitives needed beyond the `chacha20poly1305` crate.

---

## Wire Format Alignment (Session 43)

`SigningClient` uses BearDog-aligned wire DTOs with adapter-level
translation (public API unchanged, wire format matches BearDog):

| `SigningClient` method | BearDog JSON-RPC method | Status |
|------------------------|------------------------|--------|
| `sign` / `sign_owned` | `crypto.sign_ed25519` | **RESOLVED** — `message` (base64), `key_id` (DID string) |
| `verify` / `verify_owned` | `crypto.verify_ed25519` | **RESOLVED** — `message`, `signature` (base64), `public_key` (DID string) |
| `request_attestation` | `crypto.sign_contract` | **RESOLVED** — `signer` (DID), `terms` (JSON), response mapped to `Attestation` |
| `verify_did` | (no equivalent yet) | Delegates via capability adapter — BearDog DID types present |

### DID as Public Key Identifier — RESOLVED

The `public_key` field in `crypto.verify_ed25519` accepts `did:key:` strings
(e.g. `did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK`).
This is the correct semantic: `did:key:` is a self-describing public key
encoding (multibase + multicodec + raw Ed25519 bytes). BearDog resolves
`did:key:` → raw Ed25519 public key transparently during verification.

This is preferable to sending raw key bytes because:
1. DID strings are the ecosystem's canonical identity representation
2. No separate `crypto.resolve_did` round-trip is needed
3. The encoding is self-describing (includes algorithm identifier)
4. Any provider that implements `crypto.verify_ed25519` can parse `did:key:`

rhizoCrypt sends DID strings in `key_id` (sign) and `public_key` (verify)
fields consistently. This gap is formally closed.

### Type-Level Enforcement (Session 48)

The `Did` newtype now includes:
- `debug_assert` in `Did::new()` that catches non-`did:` strings in dev/test
- `Did::is_well_formed()` for runtime validation (`did:<method>:<id>`)

Wire DTOs use `String` where the field must match an external service's
schema (e.g. `CryptoSignContractResponse::public_key`). The `SigningClient`
public API exclusively takes `&Did`, enforcing DID semantics at the
application boundary.

### Remaining Evolution

- **Vertex signature verification on retrieval**: Optional verification
  of stored vertex signatures when retrieving from DAG or computing
  Merkle roots.

---

## References

- wateringHole `PRIMAL_RESPONSIBILITY_MATRIX.md` — BearDog OWNS crypto
- wateringHole `PRIMAL_IPC_PROTOCOL.md` v3.1 — `crypto.sign` discovery pattern
- wateringHole `CAPABILITY_WIRE_STANDARD.md` — identity key + ionic bond signing
- wateringHole `SPRING_COORDINATION_AND_VALIDATION.md` — provenance trio routing
- BearDog `ionic_bond.rs` — contract and bond wire types
- BearDog `primal_signing.rs` — Ed25519 primal identity key derivation
