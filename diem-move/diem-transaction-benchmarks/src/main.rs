use std::env;

use diem_transaction_benchmarks::p2p_transfer;


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: program_name <contract_name>");
        return;
    }
    
    let _contract = match args.get(1).map(|s| s.as_str()) {
        Some("p2p_transfer_parallel") => p2p_transfer::p2p_transfer_exec_parallel(),
        Some("replay_erc20_transfer") => p2p_transfer::replay_erc20_transfer(),
        _ => {
            println!("Error: Unknown contract name!");
            return; 
        }
    };
    
}