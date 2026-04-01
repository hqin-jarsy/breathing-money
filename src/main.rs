use breathing_money::chain::ChainState;
use breathing_money::miner::{Miner, create_transfer};
use breathing_money::crypto::{KeyPairPQ, Address};
use breathing_money::types::*;
use std::io::{self, Write};

fn main() {
    println!("╔══════════════════════════════════════════╗");
    println!("║     Breathing Money v0.1 Prototype       ║");
    println!("║  PoW L1 — Adaptive Monetary Base Layer   ║");
    println!("║  No sale. No premine. Research draft.     ║");
    println!("╚══════════════════════════════════════════╝");
    println!();

    // Initialize chain
    println!("Initializing chain from genesis...");
    let mut chain = ChainState::new();
    println!("{}", chain.summary());

    // Create miner
    let miner = Miner::new();
    println!();

    // Interactive REPL
    loop {
        print!("bm> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        match parts[0] {
            "mine" => {
                let count: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
                for i in 0..count {
                    if miner.mine_one(&mut chain, &[]).is_none() {
                        eprintln!("Mining failed at block {}", i);
                        break;
                    }
                }
                println!();
            }

            "status" | "info" => {
                println!("{}", chain.summary());
            }

            "balance" => {
                let addr = if let Some(addr_hex) = parts.get(1) {
                    match Address::from_hex(addr_hex) {
                        Ok(a) => a,
                        Err(e) => {
                            eprintln!("Invalid address: {}", e);
                            continue;
                        }
                    }
                } else {
                    miner.address
                };
                let bal = chain.get_balance(&addr);
                println!(
                    "Balance of {}: {:.8} BM",
                    addr.to_hex(),
                    bal as f64 / COIN as f64
                );
            }

            "send" => {
                if parts.len() < 3 {
                    println!("Usage: send <address> <amount>");
                    continue;
                }
                let recipient = match Address::from_hex(parts[1]) {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!("Invalid address: {}", e);
                        continue;
                    }
                };
                let amount: f64 = match parts[2].parse() {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!("Invalid amount: {}", e);
                        continue;
                    }
                };
                let amount_units = (amount * COIN as f64) as Amount;
                let fee = COIN / 1000; // 0.001 BM default fee

                match create_transfer(&chain, &miner.keypair, &recipient, amount_units, fee) {
                    Ok(tx) => {
                        println!("Transaction created: {}", hex::encode(tx.txid()));
                        // Mine a block with this transaction
                        if miner.mine_one(&mut chain, &[tx]).is_some() {
                            println!("Transfer confirmed in next block");
                        }
                    }
                    Err(e) => eprintln!("Transfer failed: {}", e),
                }
            }

            "newaddr" => {
                let kp = KeyPairPQ::generate();
                println!("New address: {}", kp.address().to_hex());
                println!("(Note: this address is not saved. In prototype, only miner address is active.)");
            }

            "monetary" => {
                let m = &chain.monetary;
                println!("=== Monetary State ===");
                println!("Total supply:    {:.8} BM", m.total_supply as f64 / COIN as f64);
                println!("Unmined pool:    {:.8} BM", m.unmined_pool as f64 / COIN as f64);
                println!("Planned pool:    {:.8} BM", m.planned_pool as f64 / COIN as f64);
                println!("Pool ratio:      {:.4}", m.unmined_pool as f64 / m.planned_pool.max(1) as f64);
                println!("Block reward:    {:.8} BM", m.current_block_reward as f64 / COIN as f64);
                println!("Protocol month:  {}", m.current_month);
                println!("Phase:           {}", if m.current_month < PHASE1_MONTHS { "1 (fixed 4%)" } else { "2 (adaptive)" });

                let spring_val = (m.unmined_pool as f64 / m.planned_pool.max(1) as f64).powf(SPRING_K).max(SPRING_FLOOR);
                println!("Spring value:    {:.4}", spring_val);
                println!("Effective rate:  {:.4}x base", spring_val);
            }

            "help" => {
                println!("Commands:");
                println!("  mine [n]              Mine n blocks (default: 1)");
                println!("  status / info         Show chain state");
                println!("  balance [address]     Show balance (default: miner)");
                println!("  send <addr> <amount>  Send coins and mine block");
                println!("  newaddr               Generate a new address");
                println!("  monetary              Show monetary policy state");
                println!("  help                  This message");
                println!("  quit / exit           Exit");
            }

            "quit" | "exit" => {
                println!("Goodbye.");
                break;
            }

            _ => {
                println!("Unknown command: {}. Type 'help' for commands.", parts[0]);
            }
        }
    }
}
