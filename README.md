# Breathing Money

**PoW L1 with adaptive monetary base and post-quantum signatures. No sale. No premine. No launch date.**

This is a research prototype for adversarial review before any production implementation.

## What This Is

A proof-of-work blockchain where the monetary base expands and contracts in response to on-chain settlement demand, with mathematically guaranteed perpetual miner compensation. **Post-quantum from genesis** — uses ML-DSA (CRYSTALS-Dilithium, FIPS 204), no elliptic curve cryptography.

Two mechanisms ("chisels") govern the monetary policy:

1. **Supply function** — injects new coins into the unmined pool each month, driven by a 200-month moving average of on-chain transaction volume.
2. **Extraction function** — determines per-block miner reward via a spring mechanism tied to the ratio of actual vs. planned pool size.

Floor constraints guarantee the system never reaches zero issuance or zero reward.

## Current Status

**Prototype (v0.1).** Single-node, in-memory, no P2P networking. Suitable for verifying monetary policy mechanics, not for deployment.

What works:
- Genesis block with 1.1M coin seal (unspendable)
- PoW mining with ASERT difficulty adjustment
- UTXO-based transactions with ML-DSA/Dilithium post-quantum signatures
- Algorithm-agile address format (version byte from genesis)
- Monthly supply injection (Phase 1: fixed 4% + cold start bonus)
- Spring-based block reward extraction from unmined pool
- Floor compatibility constraint enforced
- Cold start bonus (2x→1x over first 12 months, applied to injection)
- Month boundary transitions
- 22 unit tests passing

What doesn't exist yet:
- P2P networking
- Persistent storage
- Phase 2 adaptive formula (activates at month 200)
- Wallet management
- RPC interface

## Build & Run

Requires Rust 1.70+.

```bash
cargo test        # 20 tests, all should pass
cargo run         # interactive REPL
```

### Commands

```
mine [n]              Mine n blocks (default: 1)
status                Show chain state
monetary              Show monetary policy state (supply, pool, reward, spring)
balance [address]     Show balance
send <addr> <amount>  Send coins (mines a block to confirm)
help                  All commands
quit                  Exit
```

### Example Session

```
bm> mine 5
Block 1 mined! Hash: 002c2c3f... Reward: 16.00000000 BM
Block 2 mined! Hash: 009236212... Reward: 16.00000000 BM
...

bm> monetary
=== Monetary State ===
Total supply:    21000000.00000000 BM
Unmined pool:    69920.00000000 BM
Planned pool:    70000.00000000 BM
Pool ratio:      0.9989
Block reward:    16.00000000 BM
Protocol month:  0
Phase:           1 (fixed 4%)
```

## Design Document

Full whitepaper: [breathing_money_v09.md](docs/breathing_money_v09.md)

中文版: [breathing_money_v09_chinese.md](docs/breathing_money_v09_chinese.md)

## Key Parameters

| Parameter | Value | Rationale |
|---|---|---|
| Initial supply | 21,000,000 BM | Starting point (not a cap) |
| Genesis seal | 1,100,000 BM | Burned, mirrors Satoshi's untouched BTC |
| Anchor rate | 4% annual | Bias toward spending over hoarding |
| Tail floor | 0.5% annual | Perpetual minimum issuance |
| Spring floor | 0.124 | Minimum reward multiplier |
| Base reward | 16 BM/block | ~4% of 21M annually |
| MA window | 200 months | Manipulation resistance |
| Target block time | 10 minutes | |
| DAA | ASERT (2-day half-life) | Minority-chain safe |
| Cold start | 2x→1x over 12 months | Applied to supply injection |

## Architecture

```
src/
├── lib.rs           Module declarations
├── main.rs          CLI REPL
├── crypto/mod.rs    SHA-256, secp256k1, addresses
├── types/mod.rs     Block, Transaction, UTXO, constants
├── monetary/mod.rs  Supply function + extraction function + floors
├── consensus/mod.rs PoW validation, ASERT DAA, month boundaries
├── chain/mod.rs     Chain state, UTXO set, block validation, genesis
├── miner/mod.rs     Block construction, mining, transfers
└── storage/mod.rs   Placeholder (in-memory for now)
```

## Theoretical Foundation

The SAE (Self-as-an-End) framework informs the design philosophy. Evaluating the mechanism does not require reading the framework. For those interested: [self-as-an-end.net](https://self-as-an-end.net)

## License

MIT

## Contact

Han Qin — han.qin.research@gmail.com
