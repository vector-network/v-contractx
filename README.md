# v-contractx

A contract framework for Vector Network projection and reconstruction.

This repository implements the contract layer described in the Vector Network blueprint and the canonical docs. The design focuses on three hard guarantees:

1. projection isolates and locks a deterministic fraction of a vector into escrow or risk state,
2. reconstruction settles that locked value exactly once unless a staged settlement rule explicitly permits multiple phases,
3. every state-changing action emits an immutable record that can be replayed.

The projection and reconstruction rules mirror the documented semantics: projection must specify a projection identifier, locked value, rule environment, settlement rule, settlement trigger, optional drain rule, and certification requirement; reconstruction must reference a prior projection, consume it exactly once unless staged settlement is explicitly allowed, prevent double reconstruction, and preserve traceability. citeturn678054view0turn678054view1turn678054view3turn678054view4

## Repository layout

- `src/` — Rust kernel, contract runtime, projection, reconstruction, certification, and record engine.
- `sdk-ts/` — TypeScript developer SDK and app-side projection UI helpers.
- `python/` — Python outcome simulation tools.
- `wasm/` — Portable contract runtime target for WASM.
- `tests/` — integration examples and policy checks.

## Core behavior

### Projection
Projection creates a `ProjectionEnvelope` that separates a source vector into:

- `remaining` — the live, spendable balance,
- `locked` — the committed stake or directed allocation,
- `escrow` — the contract-owned or risk-owned state.

The transition is deterministic and conservative: the source loses the projected fraction, and the projection envelope records the lock state and settlement metadata.

### Reconstruction
Reconstruction consumes an active projection and settles it using a declared outcome:

- `NoChange`
- `Gain`
- `Loss`
- `PartialRelease`
- `StagedRelease`

A reconstruction receipt records the result, the settlement math, the source projection id, and whether the projection is now closed or still open for another staged settlement phase.

### Contract framework
Contracts run on top of the kernel and can authorize, deny, or transform a projection lifecycle. A contract may:

- require a minimum certification score,
- allocate locked value into a task / pool / reward bucket,
- define stake rules,
- define reward and loss rules,
- define access control rules,
- emit settlement directives for reconstruction.

## Determinism rules

- No floating point is used in kernel accounting.
- Vector components are unsigned integers.
- AuthRatio is represented as a fixed-point micro-score in the inclusive range `[0, 1_000_000]`.
- Every record hash is computed from canonical serialized bytes.
- Mutable caches are rebuildable and are never authoritative.

## Quick start

### Rust

```bash
cargo test
```

### TypeScript SDK

```bash
cd sdk-ts
npm install
npm run build
```

### Python simulator

```bash
python3 python/simulate_projection.py
```

## Example projection flow

1. Submit a projection request with a source vector and lock amount.
2. Kernel validates ownership, certification, type compatibility, and zero-safety.
3. The runtime isolates the locked portion and appends a projection record.
4. A contract evaluates the risk environment and returns a settlement directive.
5. Reconstruction applies the directive once and appends the settlement record.

## Status

This is a complete framework scaffold intended to be extended into a full protocol implementation.
