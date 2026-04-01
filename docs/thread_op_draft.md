# Thread OP Draft

**Proposed title:**
`[RESEARCH] Post-quantum PoW L1 with adaptive monetary base — ML-DSA/Dilithium from genesis, no premine — attack review wanted`

---

## Body

This is a request for adversarial technical review, not a launch announcement.

I am trying to falsify three hypotheses about a PoW L1 monetary base design. A working prototype exists — Rust, 22 tests, mines blocks and processes UTXO transactions — but no chain has launched and none will without this review. The prototype exists so you can attack real consensus code, not a notebook.

---

**What this is:**
A PoW L1 where block reward adapts to a supply pool rather than following a fixed schedule, and new supply is driven by on-chain settlement volume (200-month moving average, no oracle). Post-quantum from genesis: ML-DSA/Dilithium (NIST FIPS 204), no secp256k1. No premine, no founder allocation, no early advantage.

**What this is not:**
Not a stablecoin. Not competing with Bitcoin — Bitcoin is SoV, this is an attempt at MoE base layer. Not a launch pitch. If any of the kill criteria below are met, the design is abandoned or rewritten.

---

**Three falsifiable hypotheses:**

1. On-chain settlement volume can serve as a supply signal without an oracle, with manipulation cost sufficient to deter sustained attacks.
2. A spring-function reward controller can preserve perpetual miner compensation without freezing monetary policy.
3. The closed loop (injection ↔ pool ↔ reward ↔ hashrate ↔ block timing) is stable under realistic conditions.

---

**Five kill criteria — if any of these can be demonstrated, the design fails:**

1. A profitable sustained attack: realistic self-churn or cartel scenario with positive ROI over 12+ months under the block space parameters.
2. Floor incompatibility: floor-regime extraction exceeds floor-regime injection despite satisfying the stated constraint — at month boundaries, under rounding, or with discrete timing.
3. No acceptable DAA: under shared-hash (SHA-256) conditions, no difficulty adjustment algorithm can maintain stable block times during realistic hashrate migration.
4. Complexity not worth it: simple fixed tail emission (Monero-style) provides equivalent properties with less attack surface.
5. Signal irreparably noisy: on-chain volume, even with 200-month smoothing, carries no usable information about real settlement demand.

---

**Scope:**
This thread is asking whether the mechanism works. Questions about payment UX, merchant adoption, L2 design, and whether inflationary money is philosophically right are out of scope — not because they don't matter, but because a broken mechanism makes them irrelevant.

---

**The three points I believe are weakest:**

1. **The supply signal.** On-chain volume is polluted by wallet behavior, exchange batching, and L2 settlement patterns. I have no proof that 200-month smoothing is sufficient to extract a usable signal from this noise. This is the part I am least confident about.

2. **Collective manipulation resistance.** The prisoner's dilemma argument — that miners won't inflate volume because dilution hurts them too — is an informal incentive argument, not a proved Nash equilibrium. A well-capitalized cartel with external monetization (e.g., selling supply signal futures) might find a way to profit. I have not closed this attack surface.

3. **Cold start.** The first 12 months are the highest-risk period: low hashrate, low volume, high pool-ratio volatility. The cold start bonus (2× injection decay) is designed to help but it has not been tested under realistic adversarial conditions.

These are the three I'm least confident about — but I hold every other point in this whitepaper with equal tentativeness. Any claim can be attacked. Any assumption can be falsified. That is the point of posting this here rather than launching it.

---

**Documents:**
- Whitepaper (v0.9): https://github.com/hqin-jarsy/breathing-money/blob/v0.9-release/docs/breathing_money_v09.md
- Consensus math appendix: https://github.com/hqin-jarsy/breathing-money/blob/v0.9-release/docs/consensus_math_appendix.md
- GitHub prototype: https://github.com/hqin-jarsy/breathing-money
- Errata log: https://github.com/hqin-jarsy/breathing-money/blob/v0.9-release/docs/errata.md

---

*"The seed may be bad. The point is to find out cheaply, in public, before any launch."*

---

## Word count note

Body above: ~520 words (within 300-600 target).

## Author notes for review

- The "post-quantum from genesis" angle is the news hook (Google's March 30 paper on secp256k1 vulnerability). Lead with it in the title but don't let it dominate the thread — the mechanism review is the actual request.
- "The three points I believe are weakest" section is load-bearing. It signals intellectual honesty and invites the right attacks. Do not soften it.
- The kill criteria are stated in falsifiable terms. If a reviewer hits one, acknowledge immediately per response protocol.
- Links are placeholders — fill in after GitHub repo is tagged as v0.9-release.
