///! Monetary policy: the two chisels.
///!
///! First chisel (supply function): how much new supply enters the unmined pool each month.
///! Second chisel (extraction function): how much each block reward extracts from the pool.
///!
///! All amounts in base units (1 coin = 100_000_000 units).

use crate::types::*;

/// Generate simulated pre-history MA values for Phase 1.
/// Models 4% annualized growth starting from a baseline.
/// Returns 200 monthly volume values.
pub fn generate_simulated_ma() -> Vec<Amount> {
    // Baseline: assume genesis-era monthly volume is ~1% of total supply
    let baseline = GENESIS_SUPPLY / 100;
    let monthly_rate = (1.04_f64).powf(1.0 / 12.0);

    (0..MA_WINDOW)
        .map(|m| {
            let factor = monthly_rate.powi(m as i32);
            (baseline as f64 * factor) as Amount
        })
        .collect()
}

/// Compute the 200-month moving average at a given month.
/// Uses simulated data for months < 200, blends for months 200-399,
/// fully real data from month 400+.
pub fn compute_ma(
    month: u64,
    monthly_volumes: &[MonthlyRecord],
    simulated_ma: &[Amount],
) -> Amount {
    let window = MA_WINDOW as usize;

    if month < MA_WINDOW {
        // Pure Phase 1: use simulated + whatever real data we have
        let mut sum: u128 = 0;
        for i in 0..window {
            let m = month as i64 - i as i64;
            if m >= 0 {
                // Try real data first
                if let Some(rec) = monthly_volumes.iter().find(|r| r.month == m as u64) {
                    sum += rec.total_non_coinbase_output as u128;
                } else if (m as usize) < simulated_ma.len() {
                    sum += simulated_ma[m as usize] as u128;
                }
            } else {
                // Before genesis: use simulated data (index from end)
                let sim_idx = (simulated_ma.len() as i64 + m) as usize;
                if sim_idx < simulated_ma.len() {
                    sum += simulated_ma[sim_idx] as u128;
                }
            }
        }
        (sum / window as u128) as Amount
    } else {
        // Phase 2: blend or pure real
        let mut sum: u128 = 0;
        let mut count = 0usize;
        for i in 0..window {
            let target_month = month - i as u64;
            if let Some(rec) = monthly_volumes.iter().find(|r| r.month == target_month) {
                sum += rec.total_non_coinbase_output as u128;
                count += 1;
            } else if (target_month as usize) < simulated_ma.len() {
                sum += simulated_ma[target_month as usize] as u128;
                count += 1;
            }
        }
        if count == 0 { return 0; }
        (sum / count as u128) as Amount
    }
}

// ─── First Chisel: Supply Function ───

/// Calculate new supply to inject into unmined pool this month.
pub fn monthly_supply_injection(state: &MonetaryState) -> Amount {
    if state.current_month < PHASE1_MONTHS {
        // Phase 1: fixed 4% annualized + cold start bonus
        phase1_injection(state.total_supply, state.current_month)
    } else {
        // Phase 2: adaptive (cold start is long over by month 200)
        phase2_injection(state)
    }
}

fn phase1_injection(total_supply: Amount, month: u64) -> Amount {
    // total_supply * 0.04 / 12
    let base_injection = total_supply / 300;
    apply_cold_start(base_injection, month)
}

fn phase2_injection(state: &MonetaryState) -> Amount {
    let ma_current = compute_ma(
        state.current_month,
        &state.monthly_volumes,
        &state.simulated_ma,
    );

    let ma_past = if state.current_month >= MA_WINDOW {
        compute_ma(
            state.current_month - MA_WINDOW,
            &state.monthly_volumes,
            &state.simulated_ma,
        )
    } else {
        // Fallback to simulated baseline
        state.simulated_ma.first().copied().unwrap_or(1)
    };

    if ma_past == 0 {
        return phase1_injection(state.total_supply, state.current_month);
    }

    // annual_growth_rate = (MA_current / MA_200months_ago)^(12/200) - 1
    let ratio = ma_current as f64 / ma_past as f64;
    let exponent = 12.0 / MA_WINDOW as f64;
    let annual_rate = ratio.powf(exponent) - 1.0;

    // Apply tail floor
    let effective_rate = annual_rate.max(TAIL_FLOOR);

    // new_supply = total_supply * effective_rate / 12
    let injection = (state.total_supply as f64 * effective_rate / 12.0) as Amount;

    // Sanity: at least tail floor injection
    let min_injection = (state.total_supply as f64 * TAIL_FLOOR / 12.0) as Amount;
    injection.max(min_injection)
}

// ─── Second Chisel: Extraction Function ───

/// Calculate block reward for the current state.
pub fn calculate_block_reward(state: &MonetaryState) -> Amount {
    let pool_ratio = if state.planned_pool == 0 {
        1.0
    } else {
        state.unmined_pool as f64 / state.planned_pool as f64
    };

    // spring(x) = x^k, clamped to spring_floor
    let spring = pool_ratio.powf(SPRING_K).max(SPRING_FLOOR);

    let reward = (BASE_REWARD as f64 * spring) as Amount;

    // Cannot extract more than what's in the pool
    reward.min(state.unmined_pool)
}

/// Cold start bonus: first 12 months, linear decay from 2x to 1x
fn apply_cold_start(reward: Amount, month: u64) -> Amount {
    if month >= COLD_START_MONTHS {
        return reward;
    }
    // multiplier = 1 + (12 - month) / 12
    // At month 0: 2.0x, month 6: 1.5x, month 11: ~1.08x
    let bonus_factor = 1.0 + (COLD_START_MONTHS - month) as f64 / COLD_START_MONTHS as f64;
    (reward as f64 * bonus_factor) as Amount
}

/// Update planned pool trajectory (4% annual from genesis).
/// Models the expected pool size if the system follows 4% growth perfectly.
/// Each month: inject (total_supply * 4% / 12), extract (base_reward * blocks_per_month).
/// At the START of each month (before blocks are mined), pool = prior pool + new injection.
pub fn compute_planned_pool(month: u64) -> Amount {
    let monthly_injection_rate = ANCHOR_RATE / 12.0;
    let monthly_extraction = BASE_REWARD as f64 * BLOCKS_PER_MONTH as f64;

    let mut pool = 0.0_f64;
    let mut supply = GENESIS_SUPPLY as f64;

    for m in 0..=month {
        // Inject this month's supply
        let injection = supply * monthly_injection_rate;
        pool += injection;
        supply += injection;

        // Only extract for completed months (not the current month, since
        // we want pool state at the START of this month)
        if m < month {
            pool -= monthly_extraction.min(pool);
        }
    }

    // Ensure planned_pool is never zero (avoid division by zero in spring)
    (pool as Amount).max(1)
}

/// Initialize monetary state at genesis
pub fn genesis_monetary_state() -> MonetaryState {
    let simulated_ma = generate_simulated_ma();
    let planned_pool = compute_planned_pool(0);

    // First month's injection goes into the pool at genesis
    let first_injection = GENESIS_SUPPLY / 300; // 4% / 12

    MonetaryState {
        total_supply: GENESIS_SUPPLY,
        unmined_pool: first_injection,
        planned_pool,
        current_block_reward: calculate_block_reward_raw(first_injection, planned_pool, 0),
        current_month: 0,
        monthly_volumes: Vec::new(),
        simulated_ma,
    }
}

/// Raw block reward calculation (for initialization before full state exists)
fn calculate_block_reward_raw(unmined_pool: Amount, planned_pool: Amount, _month: u64) -> Amount {
    let pool_ratio = if planned_pool == 0 {
        1.0
    } else {
        unmined_pool as f64 / planned_pool as f64
    };
    let spring = pool_ratio.powf(SPRING_K).max(SPRING_FLOOR);
    let reward = (BASE_REWARD as f64 * spring) as Amount;
    reward.min(unmined_pool)
}

/// Advance monetary state to next month.
/// Called at month boundary (every BLOCKS_PER_MONTH blocks).
pub fn advance_month(state: &mut MonetaryState, month_volume: Amount, blocks_in_month: u64) {
    // Record this month's volume
    state.monthly_volumes.push(MonthlyRecord {
        month: state.current_month,
        total_non_coinbase_output: month_volume,
        block_count: blocks_in_month,
        first_block_height: 0, // filled by chain
        last_block_height: 0,  // filled by chain
    });

    // Advance month
    state.current_month += 1;

    // Inject new supply
    let injection = monthly_supply_injection(state);
    state.total_supply += injection;
    state.unmined_pool += injection;

    // Update planned pool
    state.planned_pool = compute_planned_pool(state.current_month);

    // Update block reward for new month
    state.current_block_reward = calculate_block_reward(state);
}

/// Process block extraction: deduct reward from pool
pub fn extract_block_reward(state: &mut MonetaryState) -> Amount {
    let reward = state.current_block_reward.min(state.unmined_pool);
    state.unmined_pool = state.unmined_pool.saturating_sub(reward);
    reward
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase1_injection() {
        // Month 0: 21M * 0.04 / 12 * 2.0 (cold start) = 140,000 coins
        let injection = phase1_injection(GENESIS_SUPPLY, 0);
        let expected = 140_000 * COIN;
        assert_eq!(injection, expected);

        // Month 12+: 21M * 0.04 / 12 = 70,000 coins (no cold start)
        let injection_12 = phase1_injection(GENESIS_SUPPLY, 12);
        let expected_12 = 70_000 * COIN;
        assert_eq!(injection_12, expected_12);
    }

    #[test]
    fn test_floor_compatibility() {
        // Verify: floor_injection >= floor_extraction (at steady state, no cold start)
        let floor_inj = (GENESIS_SUPPLY as f64) * TAIL_FLOOR / 12.0;
        let floor_ext = BASE_REWARD as f64 * SPRING_FLOOR * BLOCKS_PER_MONTH as f64;
        assert!(
            floor_inj >= floor_ext,
            "Floor incompatible: inj={} ext={}", floor_inj, floor_ext
        );
    }

    #[test]
    fn test_cold_start_bonus() {
        let base = 70_000 * COIN; // base monthly injection
        assert_eq!(apply_cold_start(base, 0), 140_000 * COIN); // 2x at month 0
        assert_eq!(apply_cold_start(base, 12), 70_000 * COIN); // 1x at month 12
        assert_eq!(apply_cold_start(base, 100), 70_000 * COIN); // 1x well past
    }

    #[test]
    fn test_genesis_state() {
        let state = genesis_monetary_state();
        assert_eq!(state.total_supply, GENESIS_SUPPLY);
        assert!(state.unmined_pool > 0);
        assert!(state.current_block_reward > 0);
        assert_eq!(state.current_month, 0);
    }

    #[test]
    fn test_spring_floor_enforced() {
        // Even with empty pool, reward should be at least spring_floor * base_reward
        let state = MonetaryState {
            total_supply: GENESIS_SUPPLY,
            unmined_pool: 1 * COIN, // nearly empty
            planned_pool: 1_000_000 * COIN, // large planned
            current_block_reward: 0,
            current_month: 13, // past cold start
            monthly_volumes: Vec::new(),
            simulated_ma: generate_simulated_ma(),
        };
        let reward = calculate_block_reward(&state);
        // Should be capped at unmined_pool since pool is tiny
        assert!(reward <= 1 * COIN);
        assert!(reward > 0);
    }
}
