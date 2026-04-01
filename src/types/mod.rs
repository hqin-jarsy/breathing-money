use serde::{Serialize, Deserialize};
use crate::crypto::{Hash256, Address, double_sha256};

/// Smallest unit. 1 coin = 100_000_000 units (like satoshis).
pub type Amount = u64;

/// 1 coin in base units
pub const COIN: Amount = 100_000_000;

/// Initial total supply: 21_000_000 coins
pub const GENESIS_SUPPLY: Amount = 21_000_000 * COIN;

/// Genesis seal: 1_100_000 coins burned
pub const GENESIS_SEAL_AMOUNT: Amount = 1_100_000 * COIN;

/// Target block time in seconds (10 minutes)
pub const TARGET_BLOCK_TIME_SECS: u64 = 600;

/// Blocks per month (approximate: 30 days * 144 blocks/day)
pub const BLOCKS_PER_MONTH: u64 = 4_380;

/// Phase 1 duration in months
pub const PHASE1_MONTHS: u64 = 200;

/// Annual growth rate anchor (4%)
pub const ANCHOR_RATE: f64 = 0.04;

/// Tail floor: minimum annual growth rate (0.5%)
pub const TAIL_FLOOR: f64 = 0.005;

/// Base reward per block in coins (16 coins)
pub const BASE_REWARD: Amount = 16 * COIN;

/// Spring floor (minimum reward multiplier)
/// Must satisfy: GENESIS_SUPPLY * TAIL_FLOOR / 12 >= BASE_REWARD * SPRING_FLOOR * BLOCKS_PER_MONTH
/// 8,750 coins >= 16 * 0.124 * 4,380 = 8,684.16 coins. OK.
pub const SPRING_FLOOR: f64 = 0.124;

/// Cold start duration in months
pub const COLD_START_MONTHS: u64 = 12;

/// MA window in months
pub const MA_WINDOW: u64 = 200;

/// Spring exponent k
pub const SPRING_K: f64 = 1.0;

// ─── Transaction ───

/// Outpoint: reference to a specific output of a previous transaction
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OutPoint {
    pub txid: Hash256,
    pub vout: u32,
}

/// Transaction input
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxInput {
    pub prev_output: OutPoint,
    /// Serialized public key (33 bytes compressed)
    pub pubkey: Vec<u8>,
    /// DER-encoded signature
    pub signature: Vec<u8>,
}

/// Transaction output
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxOutput {
    pub value: Amount,
    pub address: Address,
}

/// Transaction
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

impl Transaction {
    /// Compute transaction ID (double SHA-256 of serialized tx)
    pub fn txid(&self) -> Hash256 {
        let data = bincode::serialize(self).expect("tx serialization");
        double_sha256(&data)
    }

    /// Check if this is a coinbase transaction (single input with null prevout)
    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1 && self.inputs[0].prev_output.txid == [0u8; 32]
    }

    /// Total output value
    pub fn total_output(&self) -> Amount {
        self.outputs.iter().map(|o| o.value).sum()
    }

    /// Hash of tx data for signing (excluding signatures)
    /// Simple approach: hash inputs' outpoints + all outputs
    pub fn sighash(&self) -> Hash256 {
        let mut data = Vec::new();
        for inp in &self.inputs {
            data.extend_from_slice(&inp.prev_output.txid);
            data.extend_from_slice(&inp.prev_output.vout.to_le_bytes());
        }
        for out in &self.outputs {
            data.extend_from_slice(&out.value.to_le_bytes());
            data.extend_from_slice(&out.address.0);
        }
        double_sha256(&data)
    }
}

// ─── Block ───

/// Block header
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: u32,
    pub prev_hash: Hash256,
    pub merkle_root: Hash256,
    pub timestamp: u64,
    pub difficulty_target: Hash256,
    pub nonce: u64,
    /// Protocol month number (0-indexed from genesis)
    pub month: u64,
    /// Block height
    pub height: u64,
}

impl BlockHeader {
    /// Hash of the block header (this is the block's identity)
    pub fn hash(&self) -> Hash256 {
        let data = bincode::serialize(self).expect("header serialization");
        double_sha256(&data)
    }

    /// Check if header hash meets difficulty target
    pub fn meets_target(&self) -> bool {
        let hash = self.hash();
        hash_below_target(&hash, &self.difficulty_target)
    }
}

/// Full block
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn hash(&self) -> Hash256 {
        self.header.hash()
    }

    /// Compute merkle root of transactions
    pub fn compute_merkle_root(&self) -> Hash256 {
        let txids: Vec<Hash256> = self.transactions.iter().map(|tx| tx.txid()).collect();
        merkle_root(&txids)
    }
}

/// Simple merkle root computation
pub fn merkle_root(hashes: &[Hash256]) -> Hash256 {
    if hashes.is_empty() {
        return [0u8; 32];
    }
    if hashes.len() == 1 {
        return hashes[0];
    }

    let mut level = hashes.to_vec();
    while level.len() > 1 {
        if level.len() % 2 != 0 {
            let last = *level.last().unwrap();
            level.push(last);
        }
        let mut next = Vec::new();
        for pair in level.chunks(2) {
            let mut combined = Vec::with_capacity(64);
            combined.extend_from_slice(&pair[0]);
            combined.extend_from_slice(&pair[1]);
            next.push(double_sha256(&combined));
        }
        level = next;
    }
    level[0]
}

/// Compare hash against target (hash must be numerically less than target)
pub fn hash_below_target(hash: &Hash256, target: &Hash256) -> bool {
    // Compare as big-endian 256-bit numbers
    for i in 0..32 {
        if hash[i] < target[i] { return true; }
        if hash[i] > target[i] { return false; }
    }
    true // equal counts as meeting target
}

// ─── Chain State Tracking ───

/// Monthly volume record for MA calculation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonthlyRecord {
    pub month: u64,
    pub total_non_coinbase_output: Amount,
    pub block_count: u64,
    pub first_block_height: u64,
    pub last_block_height: u64,
}

/// Global chain monetary state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonetaryState {
    /// Current total supply (all coins ever created, including sealed)
    pub total_supply: Amount,
    /// Unmined pool (available for extraction by miners)
    pub unmined_pool: Amount,
    /// Planned pool (4% trajectory prediction from genesis)
    pub planned_pool: Amount,
    /// Current block reward (updated monthly)
    pub current_block_reward: Amount,
    /// Current protocol month
    pub current_month: u64,
    /// Monthly volume records for MA
    pub monthly_volumes: Vec<MonthlyRecord>,
    /// Simulated pre-history MA values (for Phase 1)
    pub simulated_ma: Vec<Amount>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_consistency() {
        // Floor compatibility: total_supply * tail_floor / 12 >= base_reward * spring_floor * blocks_per_month
        let floor_injection = (GENESIS_SUPPLY as f64) * TAIL_FLOOR / 12.0;
        let floor_extraction = (BASE_REWARD as f64) * SPRING_FLOOR * (BLOCKS_PER_MONTH as f64);
        assert!(
            floor_injection >= floor_extraction,
            "Floor compatibility violated: injection {} < extraction {}",
            floor_injection, floor_extraction
        );
    }

    #[test]
    fn test_hash_below_target() {
        let easy_target = [0xFF; 32];
        let hash = [0x00; 32];
        assert!(hash_below_target(&hash, &easy_target));

        let hard_target = [0x00; 32];
        let hash2 = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert!(!hash_below_target(&hash2, &hard_target));
    }

    #[test]
    fn test_merkle_root_single() {
        let h = [0xAB; 32];
        assert_eq!(merkle_root(&[h]), h);
    }
}
