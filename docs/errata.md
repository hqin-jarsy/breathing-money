# Breathing Money: Errata and Strongest Objections Log

All post-publication corrections and responses to strong objections go here.
The whitepaper and prototype code are version-frozen at the v0.9 release tag.
This log is the only place where corrections appear after posting.

Reviewers should always check this log alongside the main document.

---

## Errata

### E-001 · spring_floor corrected from 0.125 to 0.124

**Discovered by:** Prototype implementation
**Date:** 2026-03-31
**Status:** Corrected in whitepaper v0.9 and prototype code

**Error:** The whitepaper originally stated `spring_floor ≤ 0.125`. This value is wrong.

**Calculation:**

```
Floor injection:   21,000,000 coins × 0.005 / 12  = 8,750 coins/month
Floor extraction:  16 coins/block × 0.125 × 4,380  = 8,760 coins/month
```

At spring_floor = 0.125, floor extraction (8,760) exceeds floor injection (8,750). The compatibility constraint fails.

At spring_floor = 0.124:
```
Floor extraction:  16 × 0.124 × 4,380 = 8,684.16 coins/month  ≤ 8,750  ✓
```

**Correction:** `spring_floor ≤ 0.124`. Minimum per-block reward ≈ 1.98 coins (not 2.0 coins as previously stated).

**Implication:** The floor constraint is satisfied in expectation. Discrete block timing can still produce transient shortfalls at month boundaries — this is a known limitation documented in the whitepaper and appendix (Appendix §10, §11).

---

### E-002 · Cold start bonus applies to supply injection, not block reward

**Discovered by:** Prototype implementation
**Date:** 2026-03-31
**Status:** Clarified in whitepaper v0.9

**Error:** The original whitepaper wording was ambiguous about where the cold start multiplier applied. An earlier reading suggested it applied to the per-block extraction reward.

**Why that reading is wrong:** Applying the cold start bonus to the block reward drains the pool in month 0. With a base pool of 70,000 coins and a 2× block reward of 32 coins/block, extraction capacity becomes 32 × 4,380 = 140,160 coins — double the available pool. The pool exhausts in ~2,188 blocks, and the remaining ~2,192 blocks receive zero reward.

**Correct location:** The bonus applies to **supply injection**:

```
effective_supply = new_supply × (1 + max(0, (12 − m) / 12))
```

At month m=1 (post-increment in `advance_month`): multiplier = 23/12 ≈ 1.917×.
This inflates the pool to ~134,167 coins, raising pool_ratio above 1.0 and allowing spring > 1 — higher rewards without pool exhaustion.

**Note on month indexing:** `advance_month` increments the month counter before computing injection. So the "month 0" cold start bonus (2×) is never directly applied via `advance_month` — the genesis pool is seeded with the base injection (70,000 coins, no bonus). The first `advance_month` call (end of month 0 → month 1) computes injection with m=1, giving ≈ 1.917× bonus. See Appendix §5 and §10 for the worked example.

---

## Strongest Objections

*This section will be updated as substantive objections are received during review.*

*(Empty at time of posting)*

---

*This log is maintained by the author. All entries are dated and permanent — nothing is silently deleted.*
