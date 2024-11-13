use async_trait::async_trait;
use tokio::sync::RwLock;

use mithril_common::{crypto_helper::ProtocolInitializer, entities::Epoch, StdResult};
use mithril_persistence::store::{adapter::StoreAdapter, StorePruner};

use crate::services::EpochPruningTask;

type Adapter = Box<dyn StoreAdapter<Key = Epoch, Record = ProtocolInitializer>>;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
/// Store the ProtocolInitializer used for each Epoch. This is useful because
/// protocol parameters and stake distribution change over time.
pub trait ProtocolInitializerStorer: Sync + Send {
    /// Save a protocol initializer for the given Epoch.
    async fn save_protocol_initializer(
        &self,
        epoch: Epoch,
        protocol_initializer: ProtocolInitializer,
    ) -> StdResult<Option<ProtocolInitializer>>;

    /// Fetch a protocol initializer if any saved for the given Epoch.
    async fn get_protocol_initializer(
        &self,
        epoch: Epoch,
    ) -> StdResult<Option<ProtocolInitializer>>;

    /// Return the list of the N last saved protocol initializers if any.
    async fn get_last_protocol_initializer(
        &self,
        last: usize,
    ) -> StdResult<Vec<(Epoch, ProtocolInitializer)>>;
}
/// Implementation of the ProtocolInitializerStorer
pub struct ProtocolInitializerStore {
    adapter: RwLock<Adapter>,
    retention_limit: Option<usize>,
}

impl ProtocolInitializerStore {
    /// Create a new ProtocolInitializerStore.
    pub fn new(adapter: Adapter, retention_limit: Option<usize>) -> Self {
        Self {
            adapter: RwLock::new(adapter),
            retention_limit,
        }
    }
}

#[async_trait]
impl EpochPruningTask for ProtocolInitializerStore {
    fn pruned_data(&self) -> &'static str {
        "Protocol initializer"
    }

    async fn prune(&self, _epoch: Epoch) -> StdResult<()> {
        mithril_persistence::store::StorePruner::prune(self).await
    }
}

#[async_trait]
impl StorePruner for ProtocolInitializerStore {
    type Key = Epoch;
    type Record = ProtocolInitializer;

    fn get_adapter(
        &self,
    ) -> &RwLock<Box<dyn StoreAdapter<Key = Self::Key, Record = Self::Record>>> {
        &self.adapter
    }

    fn get_max_records(&self) -> Option<usize> {
        self.retention_limit
    }
}

#[async_trait]
impl ProtocolInitializerStorer for ProtocolInitializerStore {
    async fn save_protocol_initializer(
        &self,
        epoch: Epoch,
        protocol_initializer: ProtocolInitializer,
    ) -> StdResult<Option<ProtocolInitializer>> {
        let previous_protocol_initializer = self.adapter.read().await.get_record(&epoch).await?;
        self.adapter
            .write()
            .await
            .store_record(&epoch, &protocol_initializer)
            .await?;

        Ok(previous_protocol_initializer)
    }

    async fn get_protocol_initializer(
        &self,
        epoch: Epoch,
    ) -> StdResult<Option<ProtocolInitializer>> {
        let record = self.adapter.read().await.get_record(&epoch).await?;
        Ok(record)
    }

    async fn get_last_protocol_initializer(
        &self,
        last: usize,
    ) -> StdResult<Vec<(Epoch, ProtocolInitializer)>> {
        let records = self.adapter.read().await.get_last_n_records(last).await?;

        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use mithril_common::test_utils::fake_data;
    use mithril_persistence::store::adapter::MemoryAdapter;

    use super::*;

    fn setup_protocol_initializers(nb_epoch: u64) -> Vec<(Epoch, ProtocolInitializer)> {
        let mut values: Vec<(Epoch, ProtocolInitializer)> = Vec::new();
        for epoch in 1..=nb_epoch {
            let stake = (epoch + 1) * 100;
            let protocol_initializer = fake_data::protocol_initializer("1", stake);
            values.push((Epoch(epoch), protocol_initializer));
        }
        values
    }

    fn init_store(nb_epoch: u64, retention_limit: Option<usize>) -> ProtocolInitializerStore {
        let values = setup_protocol_initializers(nb_epoch);

        let values = if !values.is_empty() {
            Some(values)
        } else {
            None
        };
        let adapter: MemoryAdapter<Epoch, ProtocolInitializer> =
            MemoryAdapter::new(values).unwrap();
        ProtocolInitializerStore::new(Box::new(adapter), retention_limit)
    }

    #[tokio::test]
    async fn save_key_in_empty_store() {
        let protocol_initializers = setup_protocol_initializers(1);
        let store = init_store(0, None);
        let res = store
            .save_protocol_initializer(
                protocol_initializers[0].0,
                protocol_initializers[0].1.clone(),
            )
            .await
            .unwrap();

        assert!(res.is_none());
    }

    #[tokio::test]
    async fn update_protocol_initializer_in_store() {
        let protocol_initializers = setup_protocol_initializers(2);
        let store = init_store(1, None);
        let res = store
            .save_protocol_initializer(
                protocol_initializers[0].0,
                protocol_initializers[1].1.clone(),
            )
            .await
            .unwrap();

        assert!(res.is_some());
        assert_eq!(
            protocol_initializers[0].1.get_stake(),
            res.unwrap().get_stake()
        );
    }

    #[tokio::test]
    async fn get_protocol_initializer_for_empty_epoch() {
        let store = init_store(2, None);
        let res = store.get_protocol_initializer(Epoch(0)).await.unwrap();

        assert!(res.is_none());
    }

    #[tokio::test]
    async fn get_protocol_initializer_for_existing_epoch() {
        let store = init_store(2, None);
        let res = store.get_protocol_initializer(Epoch(1)).await.unwrap();

        assert!(res.is_some());
    }

    #[tokio::test]
    async fn check_retention_limit() {
        let store = init_store(3, Some(2));
        let _protocol_initializers = setup_protocol_initializers(1);

        assert!(store
            .get_protocol_initializer(Epoch(1))
            .await
            .unwrap()
            .is_some());

        // Whatever the epoch, it's the retention limit that matters.
        EpochPruningTask::prune(&store, Epoch(99)).await.unwrap();

        assert!(store
            .get_protocol_initializer(Epoch(1))
            .await
            .unwrap()
            .is_none());

        assert!(store
            .get_protocol_initializer(Epoch(2))
            .await
            .unwrap()
            .is_some());
    }
}
