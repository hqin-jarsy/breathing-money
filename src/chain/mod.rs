///! Blockchain state management: UTXO set, block validation and application.

use std::collections::HashMap;
use crate::crypto::{Hash256, Address};
use crate::types::*;
use crate::consensus::*;
use crate::monetary;

/// Unspent transaction output in the UTXO set
#[derive(Clone, Debug)]
pub struct UtxoEntry {
    pub output: TxOutput,
    pub height: u64,       // block height where this was created
    pub is_coinbase: bool,
}

/// The blockchain state
pub struct ChainState {
    /// UTXO set: OutPoint -> UtxoEntry
    pub utxo_set: HashMap<OutPoint, UtxoEntry>,
    /// Block index: hash -> (height, header)
    pub block_index: HashMap<Hash256, (u64, BlockHeader)>,
    /// Height -> block hash
    pub height_index: HashMap<u64, Hash256>,
    /// Current tip
    pub tip_hash: Hash256,
    pub tip_height: u64,
    /// Monetary state
    pub monetary: crate::types::MonetaryState,
    /// Accumulator for current month's non-coinbase tx output volume
    pub month_volume_accumulator: Amount,
    /// Blocks in current month
    pub month_block_count: u64,
    /// ASERT anchor (height, timestamp, target)
    pub asert_anchor: (u64, u64, Hash256),
}

impl ChainState {
    /// Create a new chain state and apply the genesis block
    pub fn new() -> Self {
        let genesis = create_genesis_block();
        let _genesis_hash = genesis.hash();

        let mut state = ChainState {
            utxo_set: HashMap::new(),
            block_index: HashMap::new(),
            height_index: HashMap::new(),
            tip_hash: [0u8; 32],
            tip_height: 0,
            monetary: monetary::genesis_monetary_state(),
            month_volume_accumulator: 0,
            month_block_count: 0,
            asert_anchor: (0, genesis.header.timestamp, genesis.header.difficulty_target),
        };

        // Apply genesis block directly (bypass normal validation for genesis)
        state.apply_genesis(&genesis);
        state
    }

    fn apply_genesis(&mut self, block: &Block) {
        let hash = block.hash();

        // Add outputs to UTXO set
        for tx in &block.transactions {
            let txid = tx.txid();
            for (vout, output) in tx.outputs.iter().enumerate() {
                let outpoint = OutPoint { txid, vout: vout as u32 };
                self.utxo_set.insert(outpoint, UtxoEntry {
                    output: output.clone(),
                    height: 0,
                    is_coinbase: tx.is_coinbase(),
                });
            }
        }

        self.block_index.insert(hash, (0, block.header.clone()));
        self.height_index.insert(0, hash);
        self.tip_hash = hash;
        self.tip_height = 0;
        self.month_block_count = 1;
    }

    /// Get the current difficulty target for the next block
    pub fn next_target(&self) -> Hash256 {
        if self.tip_height == 0 {
            return genesis_target();
        }

        let (anchor_h, anchor_ts, ref anchor_target) = self.asert_anchor;
        let tip_header = &self.block_index[&self.tip_hash].1;

        asert_target(
            anchor_h,
            anchor_ts,
            anchor_target,
            self.tip_height + 1,
            tip_header.timestamp + TARGET_BLOCK_TIME_SECS,
        )
    }

    /// Validate and apply a new block to the chain
    pub fn apply_block(&mut self, block: &Block) -> Result<(), ConsensusError> {
        let expected_height = self.tip_height + 1;

        // Check height
        if block.header.height != expected_height {
            return Err(ConsensusError::BadHeight);
        }

        // Check prev_hash
        if block.header.prev_hash != self.tip_hash {
            return Err(ConsensusError::BadPrevHash);
        }

        // Check month
        let expected_month = block_month(expected_height);
        if block.header.month != expected_month {
            return Err(ConsensusError::BadMonth);
        }

        // Handle month boundary: advance monetary state
        if is_month_boundary(expected_height) {
            monetary::advance_month(
                &mut self.monetary,
                self.month_volume_accumulator,
                self.month_block_count,
            );
            self.month_volume_accumulator = 0;
            self.month_block_count = 0;
        }

        // Validate block structure (merkle root, PoW, coinbase position)
        validate_block_structure(block)?;

        // Validate and process transactions, collect fees
        let total_fees = self.validate_transactions(block)?;

        // Validate coinbase reward
        let block_reward = self.monetary.current_block_reward.min(self.monetary.unmined_pool);
        validate_coinbase_reward(block, block_reward, total_fees)?;

        // Everything valid — apply the block

        // Extract reward from pool
        monetary::extract_block_reward(&mut self.monetary);

        // Apply UTXO changes
        self.apply_utxo_changes(block);

        // Accumulate month volume (non-coinbase tx output)
        for tx in &block.transactions[1..] {
            self.month_volume_accumulator += tx.total_output();
        }
        self.month_block_count += 1;

        // Update indices
        let hash = block.hash();
        self.block_index.insert(hash, (expected_height, block.header.clone()));
        self.height_index.insert(expected_height, hash);
        self.tip_hash = hash;
        self.tip_height = expected_height;

        Ok(())
    }

    /// Validate all non-coinbase transactions and return total fees
    fn validate_transactions(&self, block: &Block) -> Result<Amount, ConsensusError> {
        let mut total_fees: Amount = 0;

        for tx in &block.transactions[1..] {
            let mut input_sum: Amount = 0;

            for input in &tx.inputs {
                // Look up UTXO
                let utxo = self.utxo_set.get(&input.prev_output).ok_or_else(|| {
                    ConsensusError::InvalidTransaction(
                        format!("UTXO not found: {}:{}", hex::encode(input.prev_output.txid), input.prev_output.vout)
                    )
                })?;

                // Coinbase maturity: coinbase outputs require 100 confirmations
                if utxo.is_coinbase {
                    let confirmations = block.header.height.saturating_sub(utxo.height);
                    if confirmations < 100 {
                        return Err(ConsensusError::InvalidTransaction(
                            "Coinbase output not mature (need 100 confirmations)".into()
                        ));
                    }
                }

                // Verify signature (ML-DSA / Dilithium)
                let sighash = tx.sighash();
                if !crate::crypto::verify_signature(&input.pubkey, &sighash, &input.signature) {
                    return Err(ConsensusError::InvalidTransaction("Signature verification failed".into()));
                }

                // Verify pubkey matches UTXO address
                let pubkey_hash = crate::crypto::sha256(&input.pubkey);
                let mut addr = [0u8; 21];
                addr[0] = crate::crypto::ADDR_VERSION_DILITHIUM;
                addr[1..].copy_from_slice(&pubkey_hash[..20]);
                if Address(addr) != utxo.output.address {
                    return Err(ConsensusError::InvalidTransaction("Pubkey does not match UTXO address".into()));
                }

                input_sum += utxo.output.value;
            }

            let output_sum = tx.total_output();
            if output_sum > input_sum {
                return Err(ConsensusError::InvalidTransaction(
                    format!("Output ({}) exceeds input ({})", output_sum, input_sum)
                ));
            }

            total_fees += input_sum - output_sum;
        }

        Ok(total_fees)
    }

    /// Apply UTXO set changes for a block
    fn apply_utxo_changes(&mut self, block: &Block) {
        let height = block.header.height;

        // Remove spent UTXOs (skip coinbase inputs)
        for tx in &block.transactions[1..] {
            for input in &tx.inputs {
                self.utxo_set.remove(&input.prev_output);
            }
        }

        // Add new UTXOs
        for tx in &block.transactions {
            let txid = tx.txid();
            for (vout, output) in tx.outputs.iter().enumerate() {
                let outpoint = OutPoint { txid, vout: vout as u32 };
                self.utxo_set.insert(outpoint, UtxoEntry {
                    output: output.clone(),
                    height,
                    is_coinbase: tx.is_coinbase(),
                });
            }
        }
    }

    /// Get balance for an address
    pub fn get_balance(&self, address: &Address) -> Amount {
        self.utxo_set.values()
            .filter(|entry| entry.output.address == *address)
            .map(|entry| entry.output.value)
            .sum()
    }

    /// Get spendable UTXOs for an address
    pub fn get_utxos_for_address(&self, address: &Address) -> Vec<(OutPoint, &UtxoEntry)> {
        self.utxo_set.iter()
            .filter(|(_, entry)| entry.output.address == *address)
            .filter(|(_, entry)| {
                // Filter out immature coinbase outputs
                if entry.is_coinbase {
                    self.tip_height.saturating_sub(entry.height) >= 100
                } else {
                    true
                }
            })
            .map(|(op, entry)| (op.clone(), entry))
            .collect()
    }

    /// Chain summary for display
    pub fn summary(&self) -> ChainSummary {
        ChainSummary {
            height: self.tip_height,
            tip_hash: self.tip_hash,
            utxo_count: self.utxo_set.len(),
            total_supply: self.monetary.total_supply,
            unmined_pool: self.monetary.unmined_pool,
            planned_pool: self.monetary.planned_pool,
            current_reward: self.monetary.current_block_reward,
            current_month: self.monetary.current_month,
        }
    }
}

pub struct ChainSummary {
    pub height: u64,
    pub tip_hash: Hash256,
    pub utxo_count: usize,
    pub total_supply: Amount,
    pub unmined_pool: Amount,
    pub planned_pool: Amount,
    pub current_reward: Amount,
    pub current_month: u64,
}

impl std::fmt::Display for ChainSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Breathing Money Chain State ===")?;
        writeln!(f, "Height:         {}", self.height)?;
        writeln!(f, "Tip:            {}", hex::encode(self.tip_hash))?;
        writeln!(f, "UTXO count:     {}", self.utxo_count)?;
        writeln!(f, "Total supply:   {:.8} BM", self.total_supply as f64 / COIN as f64)?;
        writeln!(f, "Unmined pool:   {:.8} BM", self.unmined_pool as f64 / COIN as f64)?;
        writeln!(f, "Planned pool:   {:.8} BM", self.planned_pool as f64 / COIN as f64)?;
        writeln!(f, "Block reward:   {:.8} BM", self.current_reward as f64 / COIN as f64)?;
        writeln!(f, "Protocol month: {}", self.current_month)?;
        Ok(())
    }
}

// ─── Genesis Block ───

pub fn create_genesis_block() -> Block {
    let timestamp = 1700000000u64; // Fixed genesis timestamp

    // Genesis coinbase: creates initial supply
    // Outputs: (1) genesis seal to burn address, (2) remaining to unmined pool address
    let coinbase = Transaction {
        inputs: vec![TxInput {
            prev_output: OutPoint { txid: [0u8; 32], vout: 0xFFFFFFFF },
            pubkey: b"Breathing Money Genesis 2025".to_vec(),
            signature: Vec::new(),
        }],
        outputs: vec![
            // Genesis seal: 1.1M coins to unspendable address
            TxOutput {
                value: GENESIS_SEAL_AMOUNT,
                address: Address::genesis_seal(),
            },
        ],
    };

    let txs = vec![coinbase];
    let merkle = merkle_root(&txs.iter().map(|tx| tx.txid()).collect::<Vec<_>>());

    let header = BlockHeader {
        version: 1,
        prev_hash: [0u8; 32],
        merkle_root: merkle,
        timestamp,
        difficulty_target: genesis_target(),
        nonce: 0,
        month: 0,
        height: 0,
    };

    Block {
        header,
        transactions: txs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_block() {
        let genesis = create_genesis_block();
        assert_eq!(genesis.header.height, 0);
        assert_eq!(genesis.header.prev_hash, [0u8; 32]);
        assert!(genesis.transactions[0].is_coinbase());
        assert_eq!(genesis.transactions[0].outputs[0].value, GENESIS_SEAL_AMOUNT);
        assert_eq!(genesis.transactions[0].outputs[0].address, Address::genesis_seal());
    }

    #[test]
    fn test_chain_init() {
        let chain = ChainState::new();
        assert_eq!(chain.tip_height, 0);
        assert_eq!(chain.monetary.current_month, 0);
        assert!(chain.monetary.unmined_pool > 0);
    }

    #[test]
    fn test_genesis_seal_in_utxo() {
        let chain = ChainState::new();
        let seal_balance = chain.get_balance(&Address::genesis_seal());
        assert_eq!(seal_balance, GENESIS_SEAL_AMOUNT);
    }
}
