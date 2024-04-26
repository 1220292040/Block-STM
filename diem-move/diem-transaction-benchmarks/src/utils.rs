use csv;
use std::error::Error;
use std::fs::File;
use csv::ReaderBuilder;
use proptest::{
    strategy::Strategy, test_runner::TestRunner
};
use proptest::strategy::ValueTree;
use language_e2e_tests::account_universe::AccountUniverse;

use crate::transactions;

pub fn concatenate_strings(str1: &str, str2: &str) -> String {
    [str1,str2].join("")
}


pub fn read_csv_with_header(file_path: &str) -> Result<Vec<(usize, usize, usize)>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = ReaderBuilder::new().from_reader(file);
    let mut data = Vec::new();

    // Read the rest of the file
    for result in rdr.records() {
        let record = result?;
        let tuple = (
            record.get(0).unwrap_or("0").parse::<usize>().unwrap_or(0),
            record.get(1).unwrap_or("0").parse::<usize>().unwrap_or(0),
            record.get(2).unwrap_or("0").parse::<usize>().unwrap_or(0),
        );
        data.push(tuple);
        // println!("{:?}",tuple);
    }
    // println!("********************len:{}****************",data.len());
    Ok(data)
}

pub fn acc_gen(account_num:usize) -> AccountUniverse{
    let mut runner = TestRunner::default();
    let acc_generate_strategy = transactions::universe_strategy_with_enough_balance(account_num);
    acc_generate_strategy.new_tree(&mut runner).expect("generate account error").current().gen_account_universe()
}