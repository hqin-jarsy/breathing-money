# Breathing Money: Consensus Math Appendix

Version 0.9 — companion to the main whitepaper.
This appendix gives reviewers the exact state equations needed to construct counterexamples.
All values are in **base units** (1 coin = 10⁸ base units) unless otherwise noted.

---

## 1. State Variables

| Symbol | Rust field | Type | Description |
|--------|-----------|------|-------------|
| S | `total_supply` | u64 (base units) | All coins ever created, including genesis seal |
| P | `unmined_pool` | u64 (base units) | Coins available for block extraction |
| P\* | `planned_pool` | u64 (base units) | 4%-trajectory reference pool (no cold start) |
| m | `current_month` | u64 | Protocol month, 0-indexed from genesis |
| r | `current_block_reward` | u64 (base units) | Fixed for all blocks in current month |
| V[m] | `monthly_volumes[m].total_non_coinbase_output` | u64 (base units) | Sum of non-coinbase tx outputs in month m |
| MA(m) | computed by `compute_ma(m, ...)` | u64 (base units) | 200-month arithmetic mean ending at month m |

---

## 2. Constants

| Name | Value | Rust constant |
|------|-------|---------------|
| S₀ | 21,000,000 × 10⁸ | `GENESIS_SUPPLY` |
| B | 4,380 blocks/month | `BLOCKS_PER_MONTH` |
| R_base | 16 × 10⁸ | `BASE_REWARD` |
| f_spring | 0.124 | `SPRING_FLOOR` |
| k | 1.0 | `SPRING_K` |
| f_tail | 0.005 | `TAIL_FLOOR` |
| W | 200 months | `MA_WINDOW` |
| N_phase1 | 200 months | `PHASE1_MONTHS` |
| N_cold | 12 months | `COLD_START_MONTHS` |

---

## 3. Month Boundary Definition

Month m spans block heights **[m × B, (m+1) × B − 1]**.

The monetary state update triggers when block at height (m+1) × B − 1 is connected —
i.e., after the last block of month m is valid, before the first block of month m+1 is mined.

Block at height h belongs to month m = ⌊h / B⌋.

---

## 4. Update Order at Month Boundary (m → m+1)

`advance_month` executes the following steps in order:

```
1. V[m] ← sum of non-coinbase output values in blocks [m×B, (m+1)×B − 1]
2. m ← m + 1
3. Δ ← supply_injection(S, m)        [Section 5]
4. S ← S + Δ
5. P ← P + Δ
6. P* ← planned_pool(m)              [Section 7]
7. r ← block_reward(P, P*)           [Section 6]
```

Steps 3–7 use the **new** m (post-increment). The reward r applies to all B blocks in month m.

---

## 5. Supply Injection — First Chisel

### Phase 1 (m < 200)

```
base = ⌊S / 300⌋                         (integer floor; = ⌊S × 0.04/12⌋)

cold_bonus = 1.0 + max(0, (12 − m) / 12.0)   (float)
           = 2.0  at m=1
           = 1.0  at m≥12

Δ = ⌊base × cold_bonus⌋                  (float multiply, truncate to u64)
```

Note: `cold_bonus` uses the **post-increment** m. At m=1 the bonus is 2×, decaying to 1× by m=12. The genesis pool is seeded separately (see Section 8) and does not receive the cold start multiplier.

### Phase 2 (m ≥ 200)

```
MA_current  = MA(m)
MA_past     = MA(m − 200)

ratio = MA_current / MA_past             (float; fallback to simulated baseline if MA_past=0)
g = max(f_tail, ratio^(12/200) − 1)     (annualized growth rate, floored at 0.5%/yr)

Δ = ⌊S × g / 12⌋                        (float multiply, truncate to u64)
Δ = max(Δ, ⌊S × f_tail / 12⌋)          (sanity floor: at least tail floor injection)
```

---

## 6. Block Reward — Second Chisel

Called once per month; result r is fixed for all blocks in the month.

```
x = P / P*                               (float; if P*=0, x=1.0)
spring = max(f_spring, x^k)             (= max(0.124, x) since k=1.0)
r = ⌊R_base × spring⌋                   (float multiply, truncate to u64)
r = min(r, P)                            (cannot exceed current pool)
```

### Per-block extraction

For each block within the month:

```
paid = min(r, P)
P ← P − paid
```

r is not recalculated mid-month. Once P < r, the miner receives P and the pool reaches 0; subsequent blocks in the same month receive 0.

---

## 7. Planned Pool Trajectory

P\*(m) is recomputed from genesis at each month boundary (no incremental state):

```
pool   ← 0.0  (float)
supply ← S₀   (float)

for i in 0..=m:
    inj    = supply × 0.04 / 12
    pool   += inj
    supply += inj
    if i < m:
        pool -= min(R_base × B, pool)     (R_base × B = 70,080 coins)

P*(m) = max(1, ⌊pool⌋)
```

P\* uses only the 4% anchor rate and base reward — no cold start bonus. During months 1–11, actual P exceeds P\* due to cold start injection, driving x > 1 and higher rewards.

---

## 8. Genesis Initialization

The genesis state (before any block is mined) is constructed as:

```
S  = S₀ = 21,000,000 × 10⁸
P  = ⌊S / 300⌋ = 70,000 × 10⁸          (base injection, no cold start)
P* = P*(0) = 70,000 × 10⁸              (same formula, no prior extraction)
x  = P / P* = 1.0
r  = ⌊R_base × max(0.124, 1.0)⌋ = 16 × 10⁸  (16 coins/block)
m  = 0
```

---

## 9. MA Computation

```
MA(m) = arithmetic mean of V[m−199], V[m−198], …, V[m]
```

- For months where real data is unavailable, simulated values are substituted (4% annualized growth from baseline = S₀/100 per month).
- Simulated data is gradually replaced as real data accumulates; fully real from month 400 onward.
- MA(m) and MA(m−200) use **non-overlapping** windows: [m−199, m] and [m−399, m−200].

---

## 10. Worked Example: Month 0 Genesis → Month 1 Transition

### State at start of month 0

```
S  = 2,100,000,000,000,000  (21,000,000 coins)
P  =     7,000,000,000,000  (70,000 coins)
P* =     7,000,000,000,000  (70,000 coins)
x  = 1.0
r  = 1,600,000,000           (16 coins/block)
m  = 0
```

### Extraction during month 0

```
Capacity: r × B = 16 × 4,380 = 70,080 coins
Pool:              70,000 coins
Shortfall:              80 coins
```

Pool exhausts after block 4,374 (4,375 blocks × 16 coins = 70,000 coins extracted).
Blocks 4,375–4,379 (5 blocks) receive 0.
**This is a concrete instance of the discrete-timing edge case.**

End of month 0: P = 0.

### advance_month(V[0], 4380) — transition to month 1

```
Step 1: V[0] recorded (≥ 0)
Step 2: m ← 1

Step 3: Injection
  base         = ⌊7,000,000,000,000,000 / 300⌋
               = 7,000,000,000,000        (70,000 coins; S unchanged since P came from genesis)
  cold_bonus   = 1.0 + (12 − 1) / 12.0 = 23/12 ≈ 1.91667
  Δ            = ⌊7,000,000,000,000 × 1.91667⌋
               = ⌊13,416,666,666,666.7⌋
               = 13,416,666,666,666       (≈ 134,166.67 coins; truncation drops 0.67 coin)

Step 4: S ← 2,100,000,000,000,000 + 13,416,666,666,666
            = 2,113,416,666,666,666       (≈ 21,134,167 coins)

Step 5: P ← 0 + 13,416,666,666,666
            = 13,416,666,666,666          (≈ 134,167 coins)

Step 6: P*(1)
  i=0: inj = 21,000,000 × 0.04/12 = 70,000 coins (float)
       pool = 70,000; supply = 21,070,000
       i < 1 → pool -= min(70,080, 70,000) = 70,000 → pool = 0
  i=1: inj = 21,070,000 × 0.04/12 ≈ 70,233.33 coins
       pool = 70,233.33; supply ≈ 21,140,233
       i == 1 (== month) → no extraction
  P*(1) = ⌊70,233.33⌋ = 70,233 × 10⁸ base units  (≈ 70,233 coins)

Step 7: Block reward for month 1
  x      = 13,416,666,666,666 / 7,023,300,000,000 ≈ 1.9103
  spring = max(0.124, 1.9103) = 1.9103
  r      = ⌊1,600,000,000 × 1.9103⌋
         = ⌊3,056,480,000⌋
         = 3,056,480,000                 (≈ 30.565 coins/block)
```

### Extraction capacity check for month 1

```
Capacity: 3,056,480,000 × 4,380 = 13,387,382,400,000  (≈ 133,874 coins)
Pool:                              13,416,666,666,666  (≈ 134,167 coins)
Surplus:                                  29,284,266,666  (≈ 293 coins remain in pool)
```

Month 1 does not exhaust the pool.

---

## 11. Floor Compatibility: Exact Arithmetic

The constraint `S × f_tail / 12 ≥ R_base × f_spring × B` at genesis:

```
LHS (floor injection): 2,100,000,000,000,000 × 0.005 / 12 = 875,000,000,000  (8,750 coins)
RHS (floor extraction): 1,600,000,000 × 0.124 × 4,380    = 868,435,200,000   (≈ 8,684 coins)

8,750 ≥ 8,684  ✓
```

At f_spring = 0.125 (the prior errata value):

```
RHS: 1,600,000,000 × 0.125 × 4,380 = 876,000,000,000  (8,760 coins)
8,750 < 8,760  ✗  — constraint fails
```

This is why f_spring was corrected from 0.125 to 0.124.

The constraint is satisfied in expectation but not per-block: the month-0 shortfall (80 coins, Section 10) shows that discrete extraction can still exceed actual pool even when the floor constraint holds. The `min(r, P)` guard in `extract_block_reward` prevents pool underflow; the result is zero-reward blocks rather than a protocol error.

---

*End of appendix. Questions about specific parameter choices or alternative parameterizations: see "Where I Need Help" in the main whitepaper.*
