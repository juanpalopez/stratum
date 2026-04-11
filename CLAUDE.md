# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

**Stratum** is a toy Layer 1 blockchain built in Rust for learning purposes. The full architecture and phased implementation plan lives in `toy-l1-plan.md`. The project uses a Cargo workspace with crates named `stratum-*`.

## Workspace Layout (target structure)

```
crates/
  stratum-core/       # Phase 1 — primitives, crypto, data structures (no stratum deps)
  stratum-state/      # Phase 2 — StateDB, Merkle trie, sled storage
  stratum-network/    # Phase 3 — libp2p P2P, gossip, sync
  stratum-consensus/  # Phase 4 — PoA → Raft → Tendermint
  stratum-mempool/    # Phase 5 — tx pool, ordering, eviction
  stratum-vm/         # Phase 6 — stack-based VM, gas metering
  stratum-rpc/        # Phase 7 — JSON-RPC server (jsonrpsee)
  stratum-node/       # Phase 8 — node binary, event loop orchestrator
tools/
  stratum-cli/        # wallet, key management, tx submission
  stratum-explorer/   # ratatui TUI block explorer
tests/
  integration/        # multi-node scenario tests
  simulations/        # network partition, Byzantine fault sims
```

## Common Commands

```bash
# Build entire workspace
cargo build

# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p stratum-core

# Run a single test by name
cargo test -p stratum-core sign_verify_roundtrip

# Check without building
cargo check

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt
```

## Architecture Constraints

**Dependency direction** — only flow downward; never create cycles:
```
stratum-node → stratum-{rpc,consensus,mempool,vm,network,state}
stratum-{consensus,mempool,vm} → stratum-{core,state}
stratum-network → stratum-core
stratum-state → stratum-core
stratum-core → (no internal deps)
```

**Serialisation** — all serialised structs must use deterministic encoding (`borsh` or `bincode`). Never use `HashMap` in serialised types; use `BTreeMap`. Identical inputs must always produce identical byte output — consensus and state root correctness depend on this.

**VM determinism** — `stratum-vm` must be pure: no floats, no randomness, no I/O. Every opcode costs gas; execution halts on gas exhaustion.

**RPC is a thin adapter** — `stratum-rpc` must contain zero business logic. It calls into the node core handle and returns results.

**Consensus uses logical time** — never use wall-clock time in consensus logic. Use logical clocks or configurable tick intervals so tests are deterministic.

## Key Crates (external)

| Purpose | Crate |
|---|---|
| Signing | `ed25519-dalek` |
| Hashing | `blake3` |
| Serialisation | `borsh` |
| Storage | `sled` |
| Networking | `libp2p` |
| Async | `tokio` |
| RPC | `jsonrpsee` |
| Property tests | `proptest` |
| Logging | `tracing` |

## Phase Scope Boundaries

Each phase owns a crate and exposes only its public API contract to later phases. When implementing a phase, read the **Agent instructions** block in `.claude/plans/toy-l1-plan.md` for that phase — it defines exactly what the crate must and must not do.

## Learning Methodology

The user is learning as we build. Follow this sequence for every component within a phase:

1. **Why** — explain why the blockchain needs this component. Motivate it before defining it.
2. **Mental model** — give a concrete analogy or ASCII diagram to build intuition.
3. **How it works** — explain the mechanism (no code yet).
4. **Check-in** — ask the user a question to verify understanding before moving on. Wait for their answer.
5. **Design** — sketch the Rust types and traits together, explaining each field/method choice.
6. **Implement** — write the code, explaining non-obvious decisions inline.

Never skip the check-in. If the user's answer reveals a gap, address it before proceeding. Adjust depth based on what the user already knows — they have some background in hashing.
