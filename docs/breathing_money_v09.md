# [PRE-ANN][RESEARCH] Breathing Money — PoW Adaptive Monetary Base

**No sale. No premine. No launch date. No code yet.**
**This is a research draft for adversarial review before any implementation.**

I am trying to falsify three hypotheses:

1. On-chain settlement demand can be used as a supply signal without an oracle.
2. A reward controller can preserve perpetual miner compensation without freezing monetary policy.
3. The closed loop (supply injection ↔ extraction ↔ hashrate ↔ block timing) can be stable under realistic conditions.

If you can break any of these, that is useful.

---

## Scope of This Post

This post asks the PoW community to attack one thing: **an adaptive issuance and extraction mechanism for a PoW L1 monetary base.**

This is specifically about:
- The supply signal (what drives issuance)
- The extraction controller (what determines block reward)
- Floor guarantees (miner compensation, pool exhaustion)
- Block space policy (as monetary security)
- Difficulty adjustment assumptions

This is specifically NOT about:
- Payment UX, merchant adoption, or "will people use it as cash"
- L2 architecture, privacy, or fungibility
- Whether inflationary money is philosophically right or wrong
- Whether Bitcoin's monetary policy needs fixing

Those are real questions. They are not the questions this post is asking. If the monetary base mechanism doesn't survive adversarial review, nothing built on top of it matters.

---

## What We Do Not Know

Before the design: what we're uncertain about and where we need help.

**We do not know whether on-chain tx volume is a good supply signal.** It is the only signal we've found that is purely on-chain, requires no oracle, requires no human input, and cannot be produced without moving real value. It is imperfect — wallet behavior, exchange batching, L2 settlement, and custodian patterns all pollute it. **If you know a better signal that satisfies all of these constraints, please propose it.** Every alternative we've examined (transaction count, fee-weighted volume, UTXO set size, fee burn) has a larger attack surface or violates design principles.

**We do not know whether the 4% anchor is the right number.** It is informed by long-term US economic data (real GDP ~3.2-3.5% over 100-200 years; real stock price appreciation ~4%). We chose the upper end to bias the system toward spending over hoarding. **If 3% or 5% better serves the design goal, we want to hear why.**

**We do not know every attack vector.** The analysis below describes incentive structures we believe make sustained manipulation uneconomical. **If you can construct a profitable sustained manipulation scenario under these parameters, please share it.** That would be genuinely valuable.

**We do not know whether a 200-month MA window is the right trade-off.** Longer = harder to manipulate, slower to adapt. Shorter = more responsive, easier to game. **If you can design a shorter window with equal manipulation resistance, show us.**

---

## What Would Falsify This Design

If any of these can be demonstrated, the design should be abandoned or fundamentally rewritten:

1. **A profitable sustained attack exists.** Someone constructs a realistic self-churn or cartel scenario that produces positive ROI over 12+ months under the proposed block space and fee parameters.

2. **The floors are incompatible.** A scenario where floor-regime extraction exceeds floor-regime injection despite satisfying the stated constraint — particularly around month boundaries, reward rounding, or discrete block timing.

3. **No acceptable DAA exists.** Under shared-hash (SHA-256) conditions with monthly reward updates, no difficulty adjustment algorithm can maintain stable block times during realistic hashrate migration events.

4. **The complexity is not worth it.** A simple fixed tail emission (Monero-style) provides equivalent or better properties with less attack surface and less mechanism complexity. If the adaptive component adds risk without proportional benefit, the simpler design wins.

5. **The signal is irreparably noisy.** On-chain volume, even with 200-month smoothing, is so dominated by non-economic activity that it carries no usable information about real settlement demand.

---

## Background: Why This Exists

Bitcoin's paper was titled "A Peer-to-Peer Electronic Cash System." Its monetary policy — fixed cap, halving — made it digital gold instead. This is not a failure of adoption; it is the rational response to a deflationary currency.

This project attempts to design a monetary base layer where spending is rational rather than irrational. It does not claim to be electronic cash itself — it provides conditions for cash to emerge at L2. Whether cash actually emerges is an outcome no base layer can guarantee.

This chain does not compete with Bitcoin. Bitcoin is SoV. This is an attempt at MoE base layer. If people spend this and save Bitcoin, both work as intended.

**Theoretical foundation:** The SAE (Self-as-an-End) framework and ZFCρ mathematical proofs inform the design philosophy — particularly the idea that any formalization produces an irreducible remainder, and that systems should avoid turning their own rules into purposes. Details: [self-as-an-end.net](https://self-as-an-end.net) / [Zenodo](https://zenodo.org/search?q=self-as-an-end&f=author_name%3AHan+Qin).

**About the author:** CS PhD, read the Bitcoin whitepaper in '09, currently in blockchain (compliant tokenization). No pre-mine, no founder allocation, no early advantage. Genesis seal (see below) ensures I start mining at the same rules as everyone else.

---

## Current Working Assumptions

Everything below is a working hypothesis, not a final decision. All of it is subject to change based on this discussion and subsequent testnet results.

### Genesis

- Total initial supply: **21 million** (same as Bitcoin, but as a starting point, not a cap)
- **Genesis seal:** 1.1M coins to unspendable address. Protocol-enforced, no private key. Mirrors Satoshi's untouched ~1.1M BTC — except here it is code, not willpower. Permanently dead, visible on-chain.
- **Simulated pre-history:** Genesis contains a simulated 4% annualized tx volume growth curve as MA baseline.
- **Phase 1 (months 0–199):** Supply growth fixed at 4%. Real data accumulates; adaptive formula not yet active.
- **Phase 2 (month 200+):** Formula activates using 200 months of real data. Simulated data gradually replaced, fully real by month 400.

We know 200 months is long. This is a trade-off: robustness over responsiveness.

### First Chisel: Supply Function

Determines new supply entering the unmined pool each month.

Phase 1: `new_supply = total_supply × 0.04 / 12`

Phase 2:
```
annual_growth_rate = max(tail_floor, (MA_current / MA_200months_ago)^(12/200) - 1)
new_supply = total_supply × annual_growth_rate / 12
```

- MA = 200-month moving average of monthly non-coinbase transaction output values
- tail_floor = minimum annual growth rate (candidate: 0.5%/yr), ensuring perpetual issuance

**Signal:** Tracks L1 settlement demand, not "human economic activity" directly. Imperfect proxy, used because it is the only legitimate one under trustless constraints.

**Coinbase excluded** (prevents reflexivity). **Change outputs included** (inflate absolute volume but roughly cancel in growth rate ratio — assumption weakens when wallet/batching behavior shifts).

**No upper bound.** 200-month MA is a massive damper. No artificial cap.

**Pro-cyclicality acknowledged.** Native-unit volume rises when purchasing power falls. 200-month MA compresses this to very low frequency but does not eliminate it. This is a known limitation.

### Second Chisel: Extraction Function

Determines per-block miner reward.

**Difficulty (block timing):** Targets 10-minute blocks. Current placeholder: Bitcoin-style 2016-block retarget. **This is not a sacred choice — alternative DAA proposals are welcome, especially under shared-hash assumptions.**

### Signature Scheme: Post-Quantum from Genesis

This chain uses ML-DSA (CRYSTALS-Dilithium, FIPS 204) from genesis. **No elliptic curve cryptography. No secp256k1.**

On March 30, 2026, Google Quantum AI published resource estimates for breaking the 256-bit Elliptic Curve Discrete Logarithm Problem on secp256k1 (Babbush et al., "Securing Elliptic Curve Cryptocurrencies against Quantum Vulnerabilities"). Their result: Shor's algorithm can execute with ~1,200 logical qubits and ~90 million Toffoli gates, or ~1,450 logical qubits and ~70 million Toffoli gates. On superconducting architectures with current error rates, this translates to fewer than 500,000 physical qubits — roughly a 20x reduction from prior estimates. The paper introduces the concept of "on-spend" attacks: a quantum adversary who can derive a private key from a public key faster than a transaction confirms.

Bitcoin is already responding. BIP-360 (Pay-to-Merkle-Root) has been merged into Bitcoin's official BIP repository, removing Taproot's quantum-vulnerable keypath spend. BTQ Technologies has deployed a working implementation on testnet with Dilithium post-quantum signature opcodes. But BIP-360 only addresses long-range attacks (exposed static public keys), not on-spend attacks. And BIP-360 co-author Ethan Heilman estimates even an optimistic full migration would take Bitcoin approximately seven years.

**This chain does not yet exist. That is an advantage.** We do not need to retrofit quantum resistance onto a live network. We build it in from block 0.

Design choices:

1. **ML-DSA-65 (Dilithium3) as the genesis signature scheme.** NIST-standardized (FIPS 204), conservative security level, pure lattice-based. Public key: 1,952 bytes. Signature: 3,293 bytes. Larger than ECDSA (33 + 72 bytes), but L1 is a settlement layer — transaction size is acceptable. High-frequency payments belong on L2.

2. **Algorithm-agile address format.** Addresses carry a version byte that selects the signature verification rule. Version 0x02 = Dilithium. Future versions can introduce other PQC schemes (FN-DSA/FALCON, SLH-DSA/SPHINCS+) or hybrid schemes via soft fork, without changing the monetary layer.

3. **The signature scheme belongs to the maintainable layer, not the monetary constitution.** Block space policy, supply function, extraction function, and floor constraints are immutable. The signing algorithm is engineering — it can be upgraded as cryptographic understanding advances.

4. **Block space implications.** With ~3.3 KB signatures, a 1 MB block fits ~300 transactions vs ~3,000 with ECDSA. This is a feature, not a bug: L1 block space scarcity strengthens the supply signal (fewer transactions = each one is more economically significant) and raises the cost of volume manipulation.

**Help wanted:** What are the concrete trade-offs between ML-DSA-65, FN-DSA (FALCON-512), and SLH-DSA (SPHINCS+-128s) for a UTXO-based PoW chain? Is there a case for hybrid (classical + PQC) signatures during the transition period, or is clean PQC-only the right choice for a chain that hasn't launched yet?

**Block reward (extraction):** Monthly adjustment.

```
block_reward = base_reward × max(spring_floor, spring(actual_pool / planned_pool))
```

- base_reward ≈ 16 coins/block (calibrated: 16 × 52,560 ≈ 840K/yr ≈ 4% of 21M. Subject to simulation.)
- planned_pool = pool size predicted by 4% trajectory from genesis
- Candidate spring: `max(spring_floor, x^k)`, k > 0

### The 4% Anchor

A deliberate constitutional bias toward MoE, informed by 200 years of US economic data (real GDP 3.2-3.5%, real stock appreciation ~4%). Not an empirical law — a design choice. The spring allows the system to deviate freely; 4% is where the spring is most relaxed, not where it's forced to stay.

### Floor Compatibility Constraint

tail_floor and spring_floor must satisfy:

```
Floor injection ≥ Floor extraction

total_supply × tail_floor / 12 ≥ base_reward × spring_floor × blocks_per_month
```

With candidates (tail_floor = 0.5%/yr, base_reward = 16, ~4,380 blocks/month):

```
8,750 ≥ 70,080 × spring_floor
→ spring_floor ≤ 0.125 (minimum reward ≈ 2 coins/block)
```

Hard mathematical constraint. Guarantees floor-regime extraction never exceeds floor-regime injection. Gets easier over time as total_supply grows.

**Help wanted:** Validate this calculation. Find scenarios where floors fail despite satisfying the constraint (month boundaries, rounding, discrete timing).

### The Miner's Right

tail_floor + spring_floor = **permanent protocol-level miner compensation.** Not a subsidy — payment for the service that makes the chain exist.

Bitcoin moves miner income to fees. If fees fail, security fails. This chain guarantees positive nominal subsidy forever. It does **not** guarantee sufficient real purchasing power — if price collapses, miners may still leave. We eliminate the structural failure mode (subsidy → 0), not the market failure mode (price → 0).

### Block Space as Monetary Security

Tx volume as supply signal means block space cost = signal security cost. **Block size, fee policy, and dust rules are monetary constitution parameters, not implementation details.**

**Help wanted from miners and node operators:** What block weight, minimum fee, and dust threshold would make manipulation cost non-trivial while keeping L1 settlement affordable?

### Anti-Manipulation: Game Theory

**Individual:** Near-zero marginal cost wash trades. But resulting supply increase distributed to all miners proportionally; manipulator's gain negligible after 200-month dilution.

**Collective (prisoner's dilemma):** If all miners inflate volume: supply grows, all holdings diluted, confidence drops, price falls, difficulty rises. Nash equilibrium: don't inflate.

**Not claiming impossibility.** Claiming the incentive structure makes sustained manipulation uneconomical at scale.

**Help wanted:** Construct a profitable sustained manipulation scenario. How much does it cost to move the supply signal by 1% / 5% / 10% for 12 months under candidate block space parameters?

### Interaction Between Chisels

Negative feedback through the unmined pool. Co-determining equilibrium. Real-world coupling: reward → profitability → hashrate → block interval (until difficulty retargets) → extraction → pool → next reward.

**This closed loop requires stability simulation before mainnet.** We will not launch without it.

### Cold Start Bonus

First 12 months: `effective_supply = new_supply × (1 + max(0, (12-month)/12))`

Month 0: 2×. Month 6: 1.5×. Month 12+: 1×. Genesis code, unmodifiable.

### Extensibility

**L1:** PoW, supply function, extraction function, UTXO transfers. No smart contracts.

**L2:** Everything else. L1 reserves a minimal anchoring interface (spec after testnet).

### Protocol Maintenance

**Immutable (monetary constitution):** Supply function + floors, extraction function + floors + compatibility constraint, block space policy, genesis seal, cold start, miner's right, fork right.

**Maintainable (engineering):** P2P, storage, client, non-economic patches.

**No Foundation. No privileged maintainers.**

### Fork Rights

The constitution is immutable but not sacred. Anyone can fork. I do not oppose forks. I have no economic stake damaged by a fork.

---

## What This Is Not

Not an algorithmic stablecoin (no peg, no rebase, no oracle). Not vulnerable to subsidy death spirals (structural subsidy-to-zero risk eliminated; market risk remains). Not competing with Bitcoin. Post-quantum from genesis (ML-DSA/Dilithium, no secp256k1 — the design does not carry elliptic curve risk). Closest precedent: Monero tail emission — this goes further with adaptive emission + extraction + mathematical floor compatibility + post-quantum signatures.

---

## Where I Need Help

1. **A better supply signal** — on-chain, no oracle, no human input, manipulation cost ≥ tx volume.
2. **Spring function form and k** — constant or adaptive?
3. **Floor values** — 0.5%/yr tail, ≤0.125 spring floor. Right?
4. **Block space parameters** — weight, fee, dust. Signal security vs usability.
5. **DAA choice** — 2016-block retarget is a placeholder. Better options under shared-hash?
6. **Closed-loop stability modeling** — reward ↔ hashrate ↔ block timing ↔ extraction. If you have control systems experience, I'd like to work with you.
7. **200-month window** — too long? Show me a shorter window with equal manipulation resistance.
8. **Attack scenarios** — quantified, sustained, with cost estimates.
9. **Is the adaptive component worth its complexity vs simple fixed tail emission?**
10. **PQC signature scheme choice** — ML-DSA-65 vs FN-DSA vs SLH-DSA for UTXO PoW? Block weight implications of ~3.3 KB signatures? Case for hybrid signatures or clean PQC-only?

---

## Plan

1. This post (adversarial feedback)
2. Open reference model / simulator (public parameter search)
3. Open-source prototype
4. Testnet (stability simulation, adversarial testing, open participation)
5. Mainnet only if testnet proves the mechanism works

No timeline. No hype.

---

Bitcoin proved trustless digital money is possible. This chain asks: what if the money could breathe?
