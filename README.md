# Block-STM

This repository implements and benchmarks **Block-STM** and other baselines for the paper [Block-STM: Scaling Blockchain Execution by Turning Ordering Curse to a Performance Blessing](https://arxiv.org/abs/2203.06871).
The implementation of Block-STM has been merged on the main branch of the Diem blockchain open source code-base, see [PR](https://github.com/diem/diem/pull/10173).

Branches `block_stm` and `aptos` implement and benchmark Block-STM with Diem peer-to-peer transactions and Aptos peer-to-peer transactions, respectively.
Similarly, branches `bohm` and `litm` implement and benchmark Bohm and LiTM with Diem peer-to-peer transactions, respectively.

## Run Block-STM:
1. `./scripts/dev_setup.sh`
2. `cd diem-move/diem-transaction-benchmarks/src`
3. `cargo run --release main`

Use `taskset` commands to run experiments with different threads number. 

Set parameters (number of accounts/transactions/warmup-runs/runs) in `diem-move/diem-transaction-benchmarks/src/main.rs`.

    let acts = [2, 10, 100, 1000, 10000];
    let txns = [1000, 10000];
    let num_warmups = 2;
    let num_runs = 10;

## Run sequential baseline:
1. `./scripts/dev_setup.sh`
2. `cd diem-move/diem-transaction-benchmarks/benches`
3. `cargo bench peer_to_peer`

Set parameters (number of accounts/transactions) in `diem-move/diem-transaction-benchmarks/src/transactions.rs`.

    /// The number of accounts created by default.
    pub const DEFAULT_NUM_ACCOUNTS: usize = 100;

    /// The number of transactions created by default.
    pub const DEFAULT_NUM_TRANSACTIONS: usize = 1000;
    

---

> **Note to readers:** On December 1, 2020, the Libra Association was renamed to Diem Association. The project repos are in the process of being migrated. All projects will remain available for use here until the migration to a new GitHub Organization is complete.

<a href="https://developers.diem.com">
	<img width="200" src="./.assets/diem.png" alt="Diem Logo" />
</a>

---

[![Diem Rust Crate Documentation (main)](https://img.shields.io/badge/docs-main-59f)](https://diem.github.io/diem/)
[![License](https://img.shields.io/badge/license-Apache-green.svg)](LICENSE)
[![grcov](https://img.shields.io/badge/Coverage-grcov-green)](https://ci-artifacts.diem.com/coverage/unit-coverage/latest/index.html)
[![test history](https://img.shields.io/badge/Test-History-green)](https://ci-artifacts.diem.com/testhistory/diem/diem/auto/ci-test.yml/index.html)
[![Automated Issues](https://img.shields.io/github/issues-search?color=orange&label=Automated%20Issues&query=repo%3Adiem%2Fdiem%20is%3Aopen%20author%3Aapp%2Fgithub-actions)](https://github.com/diem/diem/issues/created_by/app/github-actions)
[![Discord chat](https://img.shields.io/discord/903339070925721652.svg?logo=discord&style=flat-square)](https://discord.gg/epNwRT2wcd)


Diem Core implements a decentralized, programmable database which provides a financial infrastructure that can empower billions of people.

## Note to Developers
* Diem Core is a prototype.
* The APIs are constantly evolving and designed to demonstrate types of functionality. Expect substantial changes before the release.
* We’ve launched a testnet that is a live demonstration of an early prototype of the Diem Blockchain software.

## Contributing

To begin contributing, [sign the CLA](https://diem.com/en-US/cla-sign/). You can learn more about contributing to the Diem project by reading our [Contribution Guide](https://developers.diem.com/docs/community/contributing) and by viewing our [Code of Conduct](https://developers.diem.com/docs/policies/code-of-conduct).

## Getting Started

### Learn About Diem
* [Welcome](https://developers.diem.com/docs/welcome-to-diem)
* [Basic Concepts](https://developers.diem.com/docs/basics/basics-txns-states)
* [Life of a Transaction](https://developers.diem.com/docs/transactions/basics-life-of-txn)
* [JSON-RPC SPEC](json-rpc/json-rpc-spec.md)

### Try Diem Core
* [My First Transaction](https://developers.diem.com/docs/tutorials/tutorial-my-first-transaction)
* [Getting Started With Move](https://diem.github.io/move/introduction.html)

### Technical Papers
* [The Diem Blockchain](https://developers.diem.com/docs/technical-papers/the-diem-blockchain-paper)
* [Move: A Language With Programmable Resources](https://developers.diem.com/docs/technical-papers/move-paper)
* [State Machine Replication in the Diem Blockchain](https://developers.diem.com/docs/technical-papers/state-machine-replication-paper)

### Blog
* [Diem: The Path Forward](https://developers.diem.com/blog/2019/06/18/the-path-forward/)

## Community

* Join us on the [Diem Discord](https://discord.gg/epNwRT2wcd) or [Discourse](https://community.diem.com).
* Ask a question on [Stack Overflow](https://stackoverflow.com/questions/tagged/diem).
* Get the latest updates to our project by signing up for our [newsletter](https://developers.diem.com/newsletter_form).

## License

Diem Core is licensed as [Apache 2.0](https://github.com/diem/diem/blob/main/LICENSE).
