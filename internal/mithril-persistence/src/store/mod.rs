//! Define a generic way to store data with the [Store Adapters][adapter], and the [StakeStorer]
//! to store stakes.

pub mod adapter;
mod stake_store;
mod store_pruner;

pub use stake_store::StakeStorer;
pub use store_pruner::StorePruner;
