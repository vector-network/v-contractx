# Vector Contract Space

A contract framework for Vector Network projection, reconstruction, certification, and deterministic settlement.

`v-contractx` implements the contract layer described in the Vector Network blueprint. The repository is built around three hard guarantees:

1. **Projection isolates and locks a deterministic fraction of a vector** into escrow, risk state, or a contract-owned environment.
2. **Reconstruction settles that locked value exactly once** unless a staged settlement rule explicitly permits multiple phases.
3. **Every state-changing action emits an immutable record** that can be replayed, verified, and synchronized across peers.

This repository is not just a contract helper library. It is the contract runtime boundary for vector-native accounting, settlement, and replayable spatial value movement.

---

## What this project is

Vector Network treats wallet and contract state as multidimensional vectors. Instead of storing only a single token balance, the system stores a vector of token dimensions, state flags, spatial context, and protocol metadata.

`v-contractx` provides the machinery for:

- deterministic projection into contract or risk environments,
- deterministic settlement and reconstruction,
- certification-gated operations,
- immutable event emission,
- contract policy enforcement,
- replay-safe record generation,
- portable execution paths for SDKs, simulation, and WASM.

The contract layer sits on top of the kernel and never replaces kernel rules. It only constrains, directs, and settles what the kernel already governs.

---

## Core guarantees

### 1. Deterministic projection
Projection must always produce the same result when given the same input state, the same rule environment, and the same lock amount. The projection step must always separate:

- the live, spendable portion,
- the locked or committed portion,
- the escrow or risk-held portion.

### 2. Single-settlement reconstruction
Reconstruction must consume a projection in a controlled and traceable way. In the default case, a projection can only be settled once. If the settlement rule explicitly supports staged release, the projection may remain open across multiple settlement phases.

### 3. Immutable record generation
Every successful state-changing action produces a canonical record. The record is part of the authoritative history. Live state is derived from history, never treated as the original source of truth.

---

## Repository layout

The repository is organized around the execution path of the protocol.

### `src/`
Rust kernel and runtime code. This is the authoritative execution layer for:

- projection,
- reconstruction,
- certification,
- contract evaluation,
- event records,
- runtime state,
- deterministic accounting.

### `sdk-ts/`
TypeScript developer SDK and app-side helpers for projection workflows, contract interactions, and UI integration.

### `python/`
Outcome simulation tools used to model projection and reconstruction behavior in a simple, dependency-light environment.

### `wasm/`
Portable contract runtime target for WASM execution where deterministic contract portability is needed.

### `tests/`
Integration tests, policy checks, and end-to-end examples for projection and reconstruction behavior.

---

## The contract model

`v-contractx` assumes the following lifecycle.

### Projection
A source vector is partially committed into a lock state. The lock can represent:

- stake,
- escrow,
- task allocation,
- directed risk,
- pool commitment,
- contract-controlled allocation.

A projection always produces a structured envelope that stores:

- projection id,
- source entity id,
- original vector,
- remaining vector,
- locked vector,
- escrow vector,
- rule environment,
- settlement rule,
- settlement trigger,
- drain policy,
- certification threshold,
- status.

### Reconstruction
A projection is settled by applying a declared outcome. The settlement may represent:

- no change,
- gain,
- loss,
- partial release,
- staged release.

Reconstruction must preserve traceability by recording:

- the originating projection id,
- the settlement outcome,
- the resulting vector,
- the settlement phase or closure state,
- the record hash,
- the causal relationship to the projection event.

### Contract framework
Contracts are programmable policy layers that can authorize, deny, or transform a projection lifecycle. They can govern:

- minimum certification requirements,
- access control,
- reward distribution,
- loss distribution,
- staking rules,
- settlement rules,
- lock duration,
- staged release behavior,
- contract-owned allocation rules,
- proof or authorization requirements.

---

## How the system works

## 1. Create or load a certified entity
An entity begins with a wallet and a vector state. The runtime validates the vector state and the certification inputs, then appends a create record.

## 2. Project part of the vector
A projection request specifies:

- which entity is being projected,
- which fraction or amount should be locked,
- which contract or rule environment applies,
- how settlement should behave,
- which drain rules, if any, apply,
- what certification threshold is required.

The kernel isolates the locked fraction and updates the derived live state to reflect the remaining balance.

## 3. Run contract logic
A contract may inspect the projection context and decide whether the projection is allowed. It may also return settlement preferences or constraints for later reconstruction.

## 4. Reconstruct or settle
The reconstruction path settles the projection based on the declared outcome. The final vector is derived from the live remainder, the locked portion, and the settlement rule.

## 5. Persist the record
The runtime emits an immutable record for every successful transition. That record can be replayed later to reconstruct the same outcome.

---

## Determinism rules

The project is designed to be deterministic across nodes, test runs, and replay passes.

### Accounting
- No floating point arithmetic is used in kernel accounting.
- Vector components are integers.
- Projection and settlement math must remain stable under replay.

### Certification
- AuthRatio is represented as a fixed-point score.
- Threshold checks are deterministic.
- Certification state is always explicit.

### Records
- Record hashes must be computed from canonical serialized bytes.
- Recomputing a record hash must return the same value for the same logical event payload.
- Mutable caches are never treated as authoritative.

### Replay
- Replaying the same valid event stream must reconstruct the same derived state.
- Invalid events must be rejected before they mutate state.
- Derived caches may be rebuilt at any time from immutable records.

---

## Main concepts

## Vector
A vector is a multidimensional state object with one component per token or dimension in the network. In practice, a vector may represent:

- wallet balance by token type,
- contract allocation,
- escrow state,
- risk exposure,
- settlement state,
- spatially meaningful state.

## Projection
Projection is the act of locking a deterministic portion of a vector into a separate tracked state.

## Escrow
Escrow is the contract-owned or protocol-owned holding area for projected value.

## Reconstruction
Reconstruction is the settlement of a projection back into a usable live state.

## Certification
Certification is the validity gate that decides whether a vector may enter restricted operations.

## Record
A record is an immutable event entry that captures a validated state change.

## Contract
A contract is a programmable policy container that evaluates whether a projection or reconstruction is allowed and how settlement behaves.

---

## Supported reconstruction outcomes

The simulation and runtime are designed to support the following logical outcomes:

- `NoChange` — the locked amount returns without gain or loss.
- `Gain` — the locked amount produces a positive settlement.
- `Loss` — some locked value is lost to the risk environment or contract outcome.
- `PartialRelease` — only part of the locked value is released.
- `StagedRelease` — the projection remains open for more than one settlement phase.

Each outcome must be handled deterministically and recorded immutably.

---

## Contract responsibilities

Contracts in `v-contractx` are responsible for expressing policy, not for overriding the kernel.

A contract may:

- require certification before projection,
- accept or deny projections,
- cap settlement gains,
- cap settlement losses,
- define reward logic,
- define stake rules,
- define lock duration,
- define release schedule,
- define access rules,
- return settlement directives,
- define whether staged settlement is allowed.

A contract may not:

- bypass the kernel validation rules,
- mutate historical records,
- invent state that was never recorded,
- skip record generation,
- make the runtime nondeterministic.

---

## Runtime architecture

The Rust runtime is structured around a kernel plus derived state.

### Kernel responsibilities
- validate requests,
- execute projection,
- execute reconstruction,
- enforce certification,
- emit records,
- maintain deterministic accounting.

### Derived state responsibilities
- store live projections,
- store entity state caches,
- store projection state caches,
- support fast reads,
- support rebuild from history.

### Event store responsibilities
- hold append-only immutable events,
- preserve replayability,
- provide the canonical history.

---

## File map

### `src/`
This is the protocol core.

Typical responsibilities include:

- `runtime.rs` — main kernel runtime, project/reconstruct flows, record creation.
- `projection.rs` — projection envelope, lock rules, drain policy, projection request handling.
- `reconstruction.rs` — settlement rules, reconstruction receipt, outcome handling.
- `certification.rs` — auth scores, thresholds, certification reports, validity checks.
- `record.rs` — record structure, canonical hashing, verification, event identity.
- `types.rs` — core domain types such as vector, wallet, entity id, region id, operation kind.
- `errors.rs` — all runtime and protocol error types.
- `contract.rs` — contract context, registry, decisions, and policy evaluation.

### `sdk-ts/`
TypeScript-facing SDK and UI support for app developers.

Typical responsibilities:

- build projection requests,
- display projected and locked balances,
- display settlement outcomes,
- interact with contracts,
- connect to RPC or node APIs,
- support browser and application integration.

### `python/`
A simple simulation harness for checking outcome logic outside the kernel.

Typical responsibilities:

- model locked vs remaining value,
- simulate gain/loss settlement,
- test phase-based settlement,
- provide readable examples for protocol behavior.

### `wasm/`
Portable runtime layer for contract execution when contracts need to run in a sandboxed, deterministic, cross-platform environment.

### `tests/`
Integration tests that verify the full lifecycle.

Typical checks:

- projection isolates the locked vector correctly,
- reconstruction settles exactly once,
- staged settlement is enforced,
- certification gating works,
- zero-vector safety is preserved,
- record generation stays deterministic,
- replay produces the same state.

---

## Build and test

### Rust
Run the Rust test suite from the repository root:

```bash
cargo test
````

If the repository uses a workspace:

```bash
cargo test --workspace
```

### Python simulation

Run the settlement simulator:

```bash
python3 python/simulate_projection.py
```

### TypeScript SDK

From the TypeScript SDK directory:

```bash
cd sdk-ts
npm install
npm run build
```

---

## Example lifecycle

### 1. Create an entity

A wallet and a vector state are submitted to the kernel. The kernel validates the input and stores the entity as certified or certifiable.

### 2. Project part of the vector

A projection request locks some portion of the vector into a contract or risk environment.

### 3. Evaluate contract logic

The contract checks whether the operation is allowed and whether the settlement environment is valid.

### 4. Settle the projection

The reconstruction logic applies the outcome and updates the live vector state.

### 5. Record the event

The runtime stores a canonical immutable record for future replay and audit.

---

## When to edit which area

### Edit `src/projection.rs`

When changing:

* projection envelope fields,
* lock behavior,
* drain handling,
* projection request validation,
* escrow representation.

### Edit `src/reconstruction.rs`

When changing:

* settlement outcomes,
* staged release logic,
* gain/loss math,
* reconstruction receipts,
* projection closure behavior.

### Edit `src/contract.rs`

When changing:

* contract policy rules,
* evaluation flow,
* settlement directives,
* access control,
* authorization logic.

### Edit `src/certification.rs`

When changing:

* validity scoring,
* thresholds,
* certification weights,
* auth ratios,
* proof logic.

### Edit `src/record.rs`

When changing:

* record structure,
* hash rules,
* serialization rules,
* canonical event representation,
* replay verification.

### Edit `python/simulate_projection.py`

When changing:

* simulation math,
* example outcomes,
* readable settlement behavior,
* staged settlement demonstrations.

### Edit `sdk-ts/`

When changing:

* developer-facing APIs,
* UI helpers,
* contract interaction flows,
* browser or app integration.

### Edit `tests/`

When adding:

* new invariants,
* regression tests,
* settlement edge cases,
* replay tests,
* policy enforcement cases.

---

## Invariants

These are non-negotiable protocol assumptions.

* No negative balances unless debt is explicitly modeled.
* No hidden state mutation.
* No recordless state changes.
* No private key exposure in shared storage.
* No undefined normalization of zero vectors.
* No nondeterministic execution in the kernel.
* No operation bypassing validation.
* No contract execution bypassing record generation.
* No mutation of historical events.
* No trust in mutable live state as source of truth.

---

## Suggested future extensions

This repository is already structured for growth. Common next steps include:

* staged settlement phases,
* slashing and penalty rules,
* multi-contract routing,
* deterministic reward distribution,
* pool accounting,
* replay compression and snapshotting,
* richer spatial state support,
* node sync and consensus layers,
* explorer and visualization tools,
* WASM-hosted contract modules,
* additional SDKs for broader language support.

---

## Status

This repository is a functional framework scaffold for the Vector Network contract layer.

It already models:

* deterministic projection,
* deterministic reconstruction,
* certification-aware execution,
* immutable record emission,
* contract policy integration,
* replay-friendly runtime structure.

It is intended to grow into a production protocol layer with stronger consensus, richer contract policies, and broader SDK support.


