///! Consensus rules: PoW validation, difficulty adjustment, month boundaries.
///!
///! Uses ASERT (absolutely scheduled exponential rising targets) for DAA,
///! which is far better suited to minority hashrate chains than Bitcoin's
///! 2016-block retarget.

use crate::types::*;
use crate::crypto::Hash256;

/// ASERT half-life in seconds (2 days = 172800s).
/// This means if blocks are 2x too fast for 2 days, difficulty doubles.
const ASERT_HALF_LIFE: f64 = 172800.0;

/// Genesis difficulty target: very easy for initial mining.
/// First byte 0x20 means about 2^(8*29) work needed — trivial.
pub fn genesis_target() -> Hash256 {
    let mut target = [0x00; 32];
    // Start very easy: target with leading 0x00, then 0xFF...
    // This means hash must start with one zero byte.
    target[0] = 0x00;
    for i in 1..32 {
        target[i] = 0xFF;
    }
    target
}

/// ASERT difficulty adjustment.
///
/// Given an anchor block (height, timestamp, target) and the current block
/// (height, timestamp), compute the new target.
///
/// Formula: new_target = anchor_target * 2^((time_delta - ideal_time_delta) / halflife)
///
/// This is the aserti3-2d algorithm used by Bitcoin Cash.
pub fn asert_target(
    anchor_height: u64,
    anchor_timestamp: u64,
    anchor_target: &Hash256,
    current_height: u64,
    current_timestamp: u64,
) -> Hash256 {
    let height_delta = current_height.saturating_sub(anchor_height) as f64;
    let time_delta = current_timestamp as f64 - anchor_timestamp as f64;
    let ideal_time = height_delta * TARGET_BLOCK_TIME_SECS as f64;

    let exponent = (time_delta - ideal_time) / ASERT_HALF_LIFE;

    // Clamp exponent to prevent overflow
    let exponent = exponent.clamp(-32.0, 32.0);

    let factor = (2.0_f64).powf(exponent);

    multiply_target(anchor_target, factor)
}

/// Multiply a 256-bit target by a floating point factor.
/// Returns a new target, clamped to valid range.
fn multiply_target(target: &Hash256, factor: f64) -> Hash256 {
    // Convert target to f64 (approximate, using top 8 bytes)
    let mut value = 0u64;
    for i in 0..8 {
        value = (value << 8) | target[i] as u64;
    }
    let new_value = (value as f64 * factor) as u64;

    // Reconstruct target
    let mut new_target = [0u8; 32];
    let bytes = new_value.to_be_bytes();
    new_target[..8].copy_from_slice(&bytes);

    // Clamp: target cannot be zero (infinite difficulty) or all-ones (zero difficulty)
    if new_target == [0u8; 32] {
        new_target[7] = 1; // minimum difficulty
    }

    // Maximum target (minimum difficulty)
    let max_target = genesis_target();
    if target_to_u64(&new_target) > target_to_u64(&max_target) {
        return max_target;
    }

    new_target
}

fn target_to_u64(target: &Hash256) -> u64 {
    let mut value = 0u64;
    for i in 0..8 {
        value = (value << 8) | target[i] as u64;
    }
    value
}

/// Determine the protocol month for a given block height.
/// Month 0 starts at height 0, each month is BLOCKS_PER_MONTH blocks.
pub fn block_month(height: u64) -> u64 {
    height / BLOCKS_PER_MONTH
}

/// Is this block the first block of a new month?
pub fn is_month_boundary(height: u64) -> bool {
    height > 0 && height % BLOCKS_PER_MONTH == 0
}

/// Validate a block header's proof of work
pub fn validate_pow(header: &BlockHeader) -> bool {
    header.meets_target()
}

/// Validate block structure
pub fn validate_block_structure(block: &Block) -> Result<(), ConsensusError> {
    // Must have at least one transaction (coinbase)
    if block.transactions.is_empty() {
        return Err(ConsensusError::NoTransactions);
    }

    // First transaction must be coinbase
    if !block.transactions[0].is_coinbase() {
        return Err(ConsensusError::NoCoinbase);
    }

    // Only first transaction can be coinbase
    for tx in &block.transactions[1..] {
        if tx.is_coinbase() {
            return Err(ConsensusError::MultipleCoinbases);
        }
    }

    // Verify merkle root
    let computed_root = block.compute_merkle_root();
    if computed_root != block.header.merkle_root {
        return Err(ConsensusError::BadMerkleRoot);
    }

    // Verify PoW
    if !validate_pow(&block.header) {
        return Err(ConsensusError::InsufficientPoW);
    }

    Ok(())
}

/// Validate coinbase reward: must not exceed allowed block reward + fees
pub fn validate_coinbase_reward(
    block: &Block,
    allowed_reward: Amount,
    total_fees: Amount,
) -> Result<(), ConsensusError> {
    let coinbase = &block.transactions[0];
    let coinbase_output: Amount = coinbase.total_output();

    if coinbase_output > allowed_reward + total_fees {
        return Err(ConsensusError::ExcessiveCoinbaseReward {
            actual: coinbase_output,
            allowed: allowed_reward + total_fees,
        });
    }

    Ok(())
}

/// Validate that the genesis seal output exists in the genesis block
pub fn validate_genesis_seal(block: &Block) -> Result<(), ConsensusError> {
    if block.header.height != 0 {
        return Ok(()); // only check genesis block
    }

    let seal_addr = crate::crypto::Address::genesis_seal();
    let has_seal = block.transactions.iter().any(|tx| {
        tx.outputs.iter().any(|out| {
            out.address == seal_addr && out.value == GENESIS_SEAL_AMOUNT
        })
    });

    if !has_seal {
        return Err(ConsensusError::MissingGenesisSeal);
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub enum ConsensusError {
    NoTransactions,
    NoCoinbase,
    MultipleCoinbases,
    BadMerkleRoot,
    InsufficientPoW,
    ExcessiveCoinbaseReward { actual: Amount, allowed: Amount },
    MissingGenesisSeal,
    InvalidTransaction(String),
    BadPrevHash,
    BadHeight,
    BadTimestamp,
    BadMonth,
}

impl std::fmt::Display for ConsensusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsensusError::NoTransactions => write!(f, "Block has no transactions"),
            ConsensusError::NoCoinbase => write!(f, "First transaction is not coinbase"),
            ConsensusError::MultipleCoinbases => write!(f, "Multiple coinbase transactions"),
            ConsensusError::BadMerkleRoot => write!(f, "Merkle root mismatch"),
            ConsensusError::InsufficientPoW => write!(f, "PoW does not meet target"),
            ConsensusError::ExcessiveCoinbaseReward { actual, allowed } =>
                write!(f, "Coinbase reward {} exceeds allowed {}", actual, allowed),
            ConsensusError::MissingGenesisSeal => write!(f, "Genesis block missing seal output"),
            ConsensusError::InvalidTransaction(msg) => write!(f, "Invalid transaction: {}", msg),
            ConsensusError::BadPrevHash => write!(f, "Previous hash mismatch"),
            ConsensusError::BadHeight => write!(f, "Invalid block height"),
            ConsensusError::BadTimestamp => write!(f, "Invalid timestamp"),
            ConsensusError::BadMonth => write!(f, "Invalid protocol month"),
        }
    }
}

impl std::error::Error for ConsensusError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_target() {
        let target = genesis_target();
        assert_eq!(target[0], 0x00);
        assert_eq!(target[1], 0xFF);
    }

    #[test]
    fn test_block_month() {
        assert_eq!(block_month(0), 0);
        assert_eq!(block_month(4379), 0);
        assert_eq!(block_month(4380), 1);
        assert_eq!(block_month(8760), 2);
    }

    #[test]
    fn test_is_month_boundary() {
        assert!(!is_month_boundary(0));
        assert!(is_month_boundary(4380));
        assert!(is_month_boundary(8760));
        assert!(!is_month_boundary(4381));
    }

    #[test]
    fn test_asert_on_target() {
        // If blocks come exactly on time, target should stay approximately the same
        let anchor_target = genesis_target();
        let new_target = asert_target(0, 0, &anchor_target, 100, 100 * 600);
        // Should be very close to anchor
        let diff = (target_to_u64(&new_target) as i128 - target_to_u64(&anchor_target) as i128).abs();
        let tolerance = target_to_u64(&anchor_target) as i128 / 100; // 1%
        assert!(diff <= tolerance, "Target drifted too much: diff={}", diff);
    }

    #[test]
    fn test_asert_slow_blocks() {
        // Blocks coming 2x too slow -> target should increase (easier)
        let anchor_target = genesis_target();
        let new_target = asert_target(0, 0, &anchor_target, 100, 100 * 1200);
        assert!(target_to_u64(&new_target) >= target_to_u64(&anchor_target));
    }
}
