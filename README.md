# Stratum

Blockchains aren't magic. Stratum builds one from scratch in Rust — keys, signatures, Merkle trees, P2P gossip, consensus, and a bytecode VM — one crate at a time.

## What it is

Stratum is a learning-oriented Layer 1 blockchain implemented from scratch in Rust. Each phase isolates a core concept so the mechanics are transparent and hackable rather than hidden behind abstraction.

| Phase | Crate | Concept |
|---|---|---|
| 1 | `stratum-core` | Cryptographic primitives, types, Merkle trees |
| 2 | `stratum-state` | World state, Merkle trie, account model |
| 3 | `stratum-network` | P2P networking with libp2p, gossip, sync |
| 4 | `stratum-consensus` | PoA → Raft → Tendermint BFT |
| 5 | `stratum-mempool` | Transaction pool, ordering, eviction |
| 6 | `stratum-vm` | Stack-based VM with gas metering |
| 7 | `stratum-rpc` | JSON-RPC interface |
| 8 | `stratum-node` | Node orchestrator, CLI, TUI explorer |

## Stack

Rust · tokio · libp2p · sled · ed25519-dalek · blake3 · borsh · jsonrpsee · ratatui

## Status

Work in progress. Follow along phase by phase.
