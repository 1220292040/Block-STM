// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod read_write_set_analyzer;
mod storage_wrapper;
mod vm_wrapper;

use crate::{
    adapter_common::{preprocess_transaction, PreprocessedTransaction},
    diem_vm::DiemVM,
    parallel_executor::vm_wrapper::DiemVMWrapper,
};
use diem_parallel_executor::{
    errors::Error,
    executor::ParallelTransactionExecutor,
    task::{Transaction as PTransaction, TransactionOutput as PTransactionOutput},
};
use diem_state_view::StateView;
use diem_types::{
    access_path::AccessPath,
    transaction::{Transaction, TransactionOutput, TransactionStatus},
    write_set::{WriteOp, WriteSet},
};
use move_core_types::vm_status::{StatusCode, VMStatus};
use rayon::prelude::*;
use std::time::Instant;

impl PTransaction for PreprocessedTransaction {
    type Key = AccessPath;
    type Value = WriteOp;
}

// Wrapper to avoid orphan rule
pub(crate) struct DiemTransactionOutput(TransactionOutput);

impl DiemTransactionOutput {
    pub fn new(output: TransactionOutput) -> Self {
        Self(output)
    }
    pub fn into(self) -> TransactionOutput {
        self.0
    }
}

impl PTransactionOutput for DiemTransactionOutput {
    type T = PreprocessedTransaction;

    fn get_writes(&self) -> Vec<(AccessPath, WriteOp)> {
        self.0.write_set().iter().cloned().collect()
    }

    /// Execution output for transactions that comes after SkipRest signal.
    fn skip_output() -> Self {
        Self(TransactionOutput::new(
            WriteSet::default(),
            vec![],
            0,
            TransactionStatus::Retry,
        ))
    }
}

pub struct ParallelDiemVM();

impl ParallelDiemVM {
    pub fn execute_block<S: StateView>(
        transactions: Vec<Transaction>,
        state_view: &S,
    ) -> Result<(Vec<TransactionOutput>, Option<Error<VMStatus>>), VMStatus> {
        // Verify the signatures of all the transactions in parallel.
        // This is time consuming so don't wait and do the checking
        // sequentially while executing the transactions.
        let signature_verified_block: Vec<PreprocessedTransaction> = transactions
            .par_iter()
            .map(|txn| preprocess_transaction::<DiemVM>(txn.clone()))
            .collect();

        match ParallelTransactionExecutor::<PreprocessedTransaction, DiemVMWrapper<S>>::new()
            .execute_transactions_parallel(state_view, signature_verified_block)
        {
            Ok(results) => Ok((
                results
                    .into_iter()
                    .map(DiemTransactionOutput::into)
                    .collect(),
                None,
            )),
            Err(err @ Error::InferencerError) | Err(err @ Error::UnestimatedWrite) => {
                let output = DiemVM::execute_block_and_keep_vm_status(transactions, state_view)?;
                Ok((
                    output
                        .into_iter()
                        .map(|(_vm_status, txn_output)| txn_output)
                        .collect(),
                    Some(err),
                ))
            }
            Err(Error::InvariantViolation) => Err(VMStatus::Error(
                StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
            )),
            Err(Error::UserError(err)) => Err(err),
        }
    }

    pub fn execute_block_tps<S: StateView>(
        transactions: Vec<Transaction>,
        state_view: &S,
    ) -> usize {
        // Verify the signatures of all the transactions in parallel.
        // This is time consuming so don't wait and do the checking
        // sequentially while executing the transactions.

        // let mut timer = Instant::now();
        let signature_verified_block: Vec<PreprocessedTransaction> = transactions
            .par_iter()
            .map(|txn| preprocess_transaction::<DiemVM>(txn.clone()))
            .collect();
        // println!("CLONE & Prologue {:?}", timer.elapsed());

        let executor = ParallelTransactionExecutor::<
            PreprocessedTransaction,
            DiemVMWrapper<S>,
        >::new();

        let timer = Instant::now();
        let useless = executor.execute_transactions_parallel(state_view, signature_verified_block);
        let exec_t = timer.elapsed();

        drop(useless);

        (transactions.len() * 1000 / exec_t.as_millis() as usize) as usize
    }
}
