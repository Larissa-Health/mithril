use std::ops::Range;

use async_trait::async_trait;

use mithril_common::crypto_helper::MKTreeNode;
use mithril_common::entities::{BlockNumber, BlockRange, CardanoTransaction, TransactionHash};
use mithril_common::StdResult;
use mithril_persistence::database::repository::CardanoTransactionRepository;

use crate::services::{TransactionStore, TransactionsRetriever};

#[async_trait]
impl TransactionStore for CardanoTransactionRepository {
    async fn get_highest_beacon(&self) -> StdResult<Option<BlockNumber>> {
        self.get_transaction_highest_block_number().await
    }

    async fn store_transactions(&self, transactions: Vec<CardanoTransaction>) -> StdResult<()> {
        self.store_transactions(transactions).await
    }

    async fn get_block_interval_without_block_range_root(
        &self,
    ) -> StdResult<Option<Range<BlockNumber>>> {
        self.get_block_interval_without_block_range_root().await
    }

    async fn get_transactions_in_range(
        &self,
        range: Range<BlockNumber>,
    ) -> StdResult<Vec<CardanoTransaction>> {
        self.get_transactions_in_range_blocks(range).await.map(|v| {
            v.into_iter()
                .map(|record| record.into())
                .collect::<Vec<CardanoTransaction>>()
        })
    }

    async fn store_block_range_roots(
        &self,
        block_ranges: Vec<(BlockRange, MKTreeNode)>,
    ) -> StdResult<()> {
        if !block_ranges.is_empty() {
            self.create_block_range_roots(block_ranges).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl TransactionsRetriever for CardanoTransactionRepository {
    async fn get_by_hashes(
        &self,
        hashes: Vec<TransactionHash>,
    ) -> StdResult<Vec<CardanoTransaction>> {
        self.get_transaction_by_hashes(hashes).await.map(|v| {
            v.into_iter()
                .map(|record| record.into())
                .collect::<Vec<CardanoTransaction>>()
        })
    }

    async fn get_by_block_ranges(
        &self,
        block_ranges: Vec<BlockRange>,
    ) -> StdResult<Vec<CardanoTransaction>> {
        self.get_transaction_by_block_ranges(block_ranges)
            .await
            .map(|v| {
                v.into_iter()
                    .map(|record| record.into())
                    .collect::<Vec<CardanoTransaction>>()
            })
    }
}
