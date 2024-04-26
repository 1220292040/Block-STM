// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use criterion::{measurement::Measurement, BatchSize, Bencher};
use diem_types::transaction::{SignedTransaction, Transaction};
use diem_vm::parallel_executor::ParallelDiemVM;
use language_e2e_tests::{
    account_universe::{log_balance_strategy, AUTransactionGen, AccountUniverseGen}, common_transactions::peer_to_peer_txn, executor::FakeExecutor, gas_costs::TXN_RESERVED
};
use proptest::{
    collection::vec, strategy::{Just, Strategy, ValueTree}, test_runner::TestRunner
};
use std::time::Instant;
use crate::utils;
use std::collections::HashMap;


/// Benchmarking support for transactions.
#[derive(Clone, Debug)]
pub struct TransactionBencher<S> {
    num_accounts: usize,
    num_transactions: usize,
    strategy: S,
}

impl TransactionBencher<()> {
    pub fn new_default() -> Self {
        Self {
            num_accounts: 0,
            num_transactions: 0,
            strategy: (),
        }
    }

     /// Runs the bencher.
     pub fn replay_parallel(
        &self,
        num_accounts: usize,
        file_path: &str,
        num_warmups: usize,
        num_runs: usize,
    ) -> Vec<usize>{
        let mut ret = Vec::new();
        let total_runs = num_warmups + num_runs;
        for i in 0..total_runs {
            let state = ParallelBenchState::setup_from_files_and_universe(file_path, universe_strategy_with_enough_balance(num_accounts));
            if i < num_warmups {
                println!("WARMUP - ignore results");
                state.execute();
            } else {
                ret.push(state.execute());
            }
        }
        ret
    }
}

impl<S> TransactionBencher<S>
where
    S: Strategy,
    S::Value: AUTransactionGen,
{
    /// The number of accounts created by default.
    pub const DEFAULT_NUM_ACCOUNTS: usize = 100;

    /// The number of transactions created by default.
    pub const DEFAULT_NUM_TRANSACTIONS: usize = 1000;

    /// Creates a new transaction bencher with default settings.
    pub fn new(strategy: S) -> Self {
        Self {
            num_accounts: Self::DEFAULT_NUM_ACCOUNTS,
            num_transactions: Self::DEFAULT_NUM_TRANSACTIONS,
            strategy,
        }
    }

    /// Sets a custom number of accounts.
    pub fn num_accounts(&mut self, num_accounts: usize) -> &mut Self {
        self.num_accounts = num_accounts;
        self
    }

    /// Sets a custom number of transactions.
    pub fn num_transactions(&mut self, num_transactions: usize) -> &mut Self {
        self.num_transactions = num_transactions;
        self
    }

    /// Runs the bencher.
    pub fn bench<M: Measurement>(&self, b: &mut Bencher<M>) {
        b.iter_batched(
            || {
                TransactionBenchState::with_size(
                    &self.strategy,
                    self.num_accounts,
                    self.num_transactions,
                )
            },
            |state| state.execute(),
            // The input here is the entire list of signed transactions, so it's pretty large.
            BatchSize::LargeInput,
        )
    }

    /// Runs the bencher.
    pub fn bench_parallel<M: Measurement>(&self, b: &mut Bencher<M>) {
        b.iter_batched(
            || {
                ParallelBenchState::with_size(
                    &self.strategy,
                    self.num_accounts,
                    self.num_transactions,
                )
            },
            |state| state.execute(),
            // The input here is the entire list of signed transactions, so it's pretty large.
            BatchSize::LargeInput,
        )
    }

    /// Runs the bencher.
    pub fn manual_parallel(
        &self,
        num_accounts: usize,
        num_txn: usize,
        num_warmups: usize,
        num_runs: usize,
    ) -> Vec<usize> {
        let mut ret = Vec::new();

        let total_runs = num_warmups + num_runs;
        for i in 0..total_runs {
            let state = ParallelBenchState::with_size(
                &self.strategy,
                num_accounts,
                num_txn,
            );
            if i < num_warmups {
                println!("WARMUP - ignore results");
                state.execute();
            } else {
                println!(
                    "RUN bencher for: num_threads = {}, \
                          block_size = {}, \
                          num_account = {}",
                    num_cpus::get(),
                    num_txn,
                    num_accounts,
                );
                ret.push(state.execute());
            }
        }

        ret
    }
}
pub struct TransactionBenchState {
    // Use the fake executor for now.
    // TODO: Hook up the real executor in the future. Here's what needs to be done:
    // 1. Provide a way to construct a write set from the genesis write set + initial balances.
    // 2. Provide a trait for an executor with the functionality required for account_universe.
    // 3. Implement the trait for the fake executor.
    // 4. Implement the trait for the real executor, using the genesis write set implemented in 1
    //    and the helpers in the execution_tests crate.
    // 5. Add a type parameter that implements the trait here and switch "executor" to use it.
    // 6. Add an enum to TransactionBencher that lets callers choose between the fake and real
    //    executors.
    executor: FakeExecutor,
    transactions: Vec<SignedTransaction>,
}

impl TransactionBenchState {
    /// Creates a new benchmark state with the given number of accounts and transactions.
    fn with_size<S>(strategy: S, num_accounts: usize, num_transactions: usize) -> Self
    where
        S: Strategy,
        S::Value: AUTransactionGen,
    {
        Self::with_universe(
            strategy,
            universe_strategy(num_accounts, num_transactions),
            num_transactions,
        )
    }

    /// Creates a new benchmark state with the given account universe strategy and number of
    /// transactions.
    fn with_universe<S>(
        strategy: S,
        universe_strategy: impl Strategy<Value = AccountUniverseGen>,
        num_transactions: usize,
    ) -> Self
    where
        S: Strategy,
        S::Value: AUTransactionGen,
    {
        let mut runner = TestRunner::default();
        let universe = universe_strategy
            .new_tree(&mut runner)
            .expect("creating a new value should succeed")
            .current();

        let mut executor = FakeExecutor::from_genesis_file();
        // Run in gas-cost-stability mode for now -- this ensures that new accounts are ignored.
        // XXX We may want to include new accounts in case they have interesting performance
        // characteristics.
        let mut universe = universe.setup_gas_cost_stability(&mut executor);

        let transaction_gens = vec(strategy, num_transactions)
            .new_tree(&mut runner)
            .expect("creating a new value should succeed")
            .current();
        let transactions = transaction_gens
            .into_iter()
            .map(|txn_gen| txn_gen.apply(&mut universe).0)
            .collect();

        Self {
            executor,
            transactions,
        }
    }
    
    fn setup_from_files_and_universe(file_path: &str,universe_strategy: impl Strategy<Value = AccountUniverseGen>)->Self{
        let mut seq_map: HashMap<usize, usize> = HashMap::new();

        let mut runner = TestRunner::default();
        let mut executor = FakeExecutor::from_genesis_file();
        let mut transactions:Vec<SignedTransaction> = Vec::new();
        let universe = universe_strategy
            .new_tree(&mut runner)
            .expect("creating a new value should succeed")
            .current();
        let universe = universe.setup_gas_cost_stability(&mut executor);
        //construct transaction
        let result = utils::read_csv_with_header(file_path);
        match result {
            Ok(data) => {
                for tuple in &data{
                    let sender = universe.get_account(tuple.0);
                    let receiver = universe.get_account(tuple.1);

                    let entry = seq_map.entry(tuple.0);
                    match entry {
                        std::collections::hash_map::Entry::Occupied(mut occupied)=>{
                            *occupied.get_mut()+=1;
                        }
                        std::collections::hash_map::Entry::Vacant(vacant) => {
                            vacant.insert(sender.sequence_number() as usize);
                        }
                    };
                    let txn = peer_to_peer_txn(
                        sender.account(), 
                        receiver.account(), 
                        seq_map[&tuple.0] as u64, 
                        1,
                    );
                    transactions.push(txn);   
                }
            },
            Err(err) =>{
                eprintln!("Error: {:?}", err);
            }
        }
        Self{
            executor,
            transactions,
        }
    }
    /// Executes this state in a single block.
    fn execute(self) {
        // The output is ignored here since we're just testing transaction performance, not trying
        // to assert correctness.
        self.executor
            .execute_block(self.transactions)
            .expect("VM should not fail to start");
    }

    /// Executes this state in a single block for tps calc.
    fn execute_block_sequential_tps(self) -> usize {
        let transaction_size = self.transactions.len();
        let timer = Instant::now();
        let useless = self.executor.execute_block_and_keep_vm_status(
            self.transactions,
        );
        let exec_t = timer.elapsed();
        drop(useless);

        (transaction_size * 1000 / exec_t.as_millis() as usize) as usize    
    }
}

/// Returns a strategy for the account universe customized for benchmarks.
fn universe_strategy(
    num_accounts: usize,
    num_transactions: usize,
) -> impl Strategy<Value = AccountUniverseGen> {
    // Multiply by 5 past the number of  to provide
    let max_balance = TXN_RESERVED * num_transactions as u64 * 5;
    let balance_strategy = log_balance_strategy(max_balance);
    AccountUniverseGen::strategy(num_accounts, balance_strategy)
}

pub fn universe_strategy_with_enough_balance(
    num_accounts: usize,
) -> impl Strategy<Value = AccountUniverseGen> {
    // provide max_balance(10^9) for every accounts
    let max_balance = 1_000_000_000;
    let balance_strategy = Just(max_balance);
    AccountUniverseGen::strategy(num_accounts, balance_strategy)
}

struct ParallelBenchState {
    bench_state: TransactionBenchState,
}

impl ParallelBenchState {
    /// Creates a new benchmark state with the given number of accounts and transactions.
    fn with_size<S>(
        strategy: S,
        num_accounts: usize,
        num_transactions: usize,
    ) -> Self
    where
        S: Strategy,
        S::Value: AUTransactionGen,
    {
        Self {
            bench_state: TransactionBenchState::with_universe(
                strategy,
                universe_strategy(num_accounts, num_transactions),
                num_transactions,
            ),
        }
    }

    fn setup_from_files_and_universe(file_path: &str,universe_strategy: impl Strategy<Value = AccountUniverseGen>) -> Self{
        Self {
            bench_state: TransactionBenchState::setup_from_files_and_universe(file_path,universe_strategy),
        }
    }

    fn execute(self) -> usize {
        let txns = self
            .bench_state
            .transactions
            .into_iter()
            .map(Transaction::UserTransaction)
            .collect();
        let state_view = self.bench_state.executor.get_state_view();
        // measured - microseconds.
        ParallelDiemVM::execute_block_tps(
            txns,
            state_view,
        )
    }
}