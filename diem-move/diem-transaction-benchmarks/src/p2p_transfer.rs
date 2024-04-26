use core::num;

use language_e2e_tests::account_universe::P2PTransferGen;
use num_cpus;
use proptest::prelude::*;

use crate::{transactions::TransactionBencher, utils};


pub fn p2p_transfer_exec_parallel(){
    // let bencher = TransactionBencher::new(any_with::<P2PTransferGen>((1_000, 1_000_000)));
    let bencher = TransactionBencher::new(any_with::<P2PTransferGen>((1, 2)));
    let acts = [200];
    let txns = [100000];
    // let acts = [2,1000,100000];
    // let txns = [1000];
    // let acts = [2];
    // let txns = [10];
    let num_warmups = 0;
    let num_runs = 3;

    let mut measurements = Vec::new();

    for block_size in txns {
        for num_accounts in acts {
            println!("parallet exec begin:{} acc",num_accounts);
            let mut times = bencher.manual_parallel(
                num_accounts,
                block_size,
                num_warmups,
                num_runs,
            );
            println!("parallet exec end:{} acc",num_accounts);
            times.sort();
            measurements.push(times);
        }
    }

    println!("CPUS = {}", num_cpus::get());

    let mut i = 0;
    for block_size in txns {
        for num_accounts in acts {
            println!(
                "PARAMS: num_account = {}, block_size = {}",
                num_accounts, block_size
            );
            println!(
                "TPS: {:?}",
                measurements[i]
            );
            let mut sum = 0;
            for m in &measurements[i] {
                sum += m;
            }
            println!(
                "AVG TPS = {:?}",
                sum / measurements[i].len()
            );
            i = i + 1;
        }
        println!();
    }
}

pub fn replay_erc20_transfer(){
    let num_warmups = 0;
    let num_runs = 3;
    let file_path = "../data/WETH_num_tx_1000000.csv";
    let bencher = TransactionBencher::new_default();
    let res = utils::read_csv_with_header(file_path);
    let mut num_accounts = 0;
    let mut block_size = 0;
    match res {
        Ok(data)=>{
            for acc_pair in &data{
                num_accounts = num_accounts.max(acc_pair.0).max(acc_pair.1);
            }
            block_size = data.len();
        }
        Err(err)=>{
            eprintln!("Error: {:?}", err);
        }
    }
    let tps = bencher.replay_parallel(300000, file_path, num_warmups, num_runs);
    let mut sum = 0;
    println!("CPUS = {}", num_cpus::get());
    println!(
        "PARAMS: num_account = {}, block_size = {}",
        num_accounts, block_size
    );
    for t in &tps{
        sum += t;
    }
    println!("TPS = {:?}", tps);
    println!(
        "AVG TPS = {:?}",
        sum / tps.len()
    );
    println!();
}