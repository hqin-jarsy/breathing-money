///! Miner: construct candidate blocks and find valid nonces.

use crate::crypto::{Address, KeyPairPQ};
use crate::types::*;
use crate::chain::ChainState;
use crate::consensus::block_month;
use std::time::{SystemTime, UNIX_EPOCH};

/// Construct a candidate block (without valid nonce yet)
pub fn build_candidate_block(
    chain: &ChainState,
    miner_address: &Address,
    pending_txs: &[Transaction],
) -> Block {
    let height = chain.tip_height + 1;
    let month = block_month(height);
    let reward = chain.monetary.current_block_reward.min(chain.monetary.unmined_pool);

    // Collect fees from pending transactions
    let mut total_fees: Amount = 0;
    let mut valid_txs: Vec<Transaction> = Vec::new();

    for tx in pending_txs {
        // Quick validation: check UTXOs exist and compute fee
        let mut input_sum: Amount = 0;
        let mut valid = true;

        for input in &tx.inputs {
            if let Some(utxo) = chain.utxo_set.get(&input.prev_output) {
                input_sum += utxo.output.value;
            } else {
                valid = false;
                break;
            }
        }

        if valid && input_sum >= tx.total_output() {
            total_fees += input_sum - tx.total_output();
            valid_txs.push(tx.clone());
        }
    }

    // Build coinbase transaction
    let coinbase = Transaction {
        inputs: vec![TxInput {
            prev_output: OutPoint { txid: [0u8; 32], vout: 0xFFFFFFFF },
            pubkey: height.to_le_bytes().to_vec(), // encode height in coinbase
            signature: Vec::new(),
        }],
        outputs: vec![TxOutput {
            value: reward + total_fees,
            address: *miner_address,
        }],
    };

    let mut all_txs = vec![coinbase];
    all_txs.extend(valid_txs);

    let merkle = crate::types::merkle_root(
        &all_txs.iter().map(|tx| tx.txid()).collect::<Vec<_>>()
    );

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let target = chain.next_target();

    let header = BlockHeader {
        version: 1,
        prev_hash: chain.tip_hash,
        merkle_root: merkle,
        timestamp,
        difficulty_target: target,
        nonce: 0,
        month,
        height,
    };

    Block {
        header,
        transactions: all_txs,
    }
}

/// Mine a block: find a valid nonce.
/// Returns the block with valid nonce, or None if max_attempts exceeded.
pub fn mine_block(
    mut block: Block,
    max_attempts: u64,
) -> Option<Block> {
    for nonce in 0..max_attempts {
        block.header.nonce = nonce;

        // Recompute merkle root only if nonce changes affect coinbase
        // (it doesn't in our design, nonce is in header)

        if block.header.meets_target() {
            return Some(block);
        }
    }
    None
}

/// Continuous mining loop: build and mine blocks
pub struct Miner {
    pub keypair: KeyPairPQ,
    pub address: Address,
}

impl Miner {
    pub fn new() -> Self {
        let keypair = KeyPairPQ::generate();
        let address = keypair.address();
        println!("Miner address: {}", address.to_hex());
        Miner { keypair, address }
    }

    pub fn from_keypair(keypair: KeyPairPQ) -> Self {
        let address = keypair.address();
        Miner { keypair, address }
    }

    /// Mine one block and apply it to the chain
    pub fn mine_one(&self, chain: &mut ChainState, pending_txs: &[Transaction]) -> Option<Block> {
        let candidate = build_candidate_block(chain, &self.address, pending_txs);
        let target_hex = hex::encode(candidate.header.difficulty_target);
        let height = candidate.header.height;

        println!("Mining block {}... (target: {}...)", height, &target_hex[..16]);

        match mine_block(candidate, u64::MAX) {
            Some(block) => {
                let hash = block.hash();
                match chain.apply_block(&block) {
                    Ok(()) => {
                        println!(
                            "Block {} mined! Hash: {}... Reward: {:.8} BM",
                            height,
                            &hex::encode(hash)[..16],
                            block.transactions[0].total_output() as f64 / COIN as f64,
                        );
                        Some(block)
                    }
                    Err(e) => {
                        eprintln!("Block {} rejected: {}", height, e);
                        None
                    }
                }
            }
            None => {
                eprintln!("Failed to mine block {}", height);
                None
            }
        }
    }
}

/// Create a simple transfer transaction
pub fn create_transfer(
    chain: &ChainState,
    sender: &KeyPairPQ,
    recipient: &Address,
    amount: Amount,
    fee: Amount,
) -> Result<Transaction, String> {
    let sender_addr = sender.address();
    let utxos = chain.get_utxos_for_address(&sender_addr);

    // Select UTXOs to cover amount + fee
    let mut selected = Vec::new();
    let mut total_input: Amount = 0;
    let needed = amount + fee;

    for (outpoint, entry) in &utxos {
        selected.push((outpoint.clone(), entry));
        total_input += entry.output.value;
        if total_input >= needed {
            break;
        }
    }

    if total_input < needed {
        return Err(format!(
            "Insufficient funds: have {:.8}, need {:.8}",
            total_input as f64 / COIN as f64,
            needed as f64 / COIN as f64,
        ));
    }

    // Build outputs
    let mut outputs = vec![TxOutput {
        value: amount,
        address: *recipient,
    }];

    // Change output
    let change = total_input - needed;
    if change > 0 {
        outputs.push(TxOutput {
            value: change,
            address: sender_addr,
        });
    }

    // Build inputs (without signatures first)
    let inputs: Vec<TxInput> = selected.iter().map(|(op, _)| TxInput {
        prev_output: op.clone(),
        pubkey: sender.pubkey_bytes().to_vec(),
        signature: Vec::new(), // placeholder
    }).collect();

    let mut tx = Transaction { inputs, outputs };

    // Sign with Dilithium
    let sighash = tx.sighash();
    let sig = sender.sign(&sighash);

    for input in &mut tx.inputs {
        input.signature = sig.clone();
    }

    Ok(tx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_candidate() {
        let chain = ChainState::new();
        let miner = Miner::new();
        let candidate = build_candidate_block(&chain, &miner.address, &[]);
        assert_eq!(candidate.header.height, 1);
        assert_eq!(candidate.header.prev_hash, chain.tip_hash);
        assert!(candidate.transactions[0].is_coinbase());
    }
}
