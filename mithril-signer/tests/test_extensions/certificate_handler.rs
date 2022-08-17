use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use mithril_common::{
    entities::{Beacon, CertificatePending, Epoch, Signer, SingleSignatures},
    BeaconProvider, BeaconProviderImpl,
};
use mithril_signer::{CertificateHandler, CertificateHandlerError};
use tokio::sync::RwLock;

pub struct FakeAggregator {
    registered_signers: RwLock<HashMap<Epoch, Vec<Signer>>>,
    beacon_provider: Arc<BeaconProviderImpl>,
}

impl FakeAggregator {
    pub fn new(beacon_provider: Arc<BeaconProviderImpl>) -> Self {
        Self {
            registered_signers: RwLock::new(HashMap::new()),
            beacon_provider,
        }
    }

    pub async fn get_registered_signers(&self, epoch: &Epoch) -> Option<Vec<Signer>> {
        let store = self.registered_signers.read().await;

        store.get(epoch).map(|s| s.clone())
    }

    async fn get_epoch(&self) -> Result<Epoch, CertificateHandlerError> {
        let epoch = self
            .beacon_provider
            .get_current_beacon()
            .await
            .map_err(|e| CertificateHandlerError::RemoteServerTechnical(e.to_string()))?
            .epoch;

        Ok(epoch)
    }
}

#[async_trait]
impl CertificateHandler for FakeAggregator {
    async fn retrieve_pending_certificate(
        &self,
    ) -> Result<Option<CertificatePending>, CertificateHandlerError> {
        let store = self.registered_signers.read().await;

        if store.is_empty() {
            return Ok(None);
        }
        let epoch = self.get_epoch().await?;
        let mut certificate_pending = CertificatePending {
            ..Default::default()
        };

        let store = self.registered_signers.read().await;
        certificate_pending.signers = store
            .get(&epoch.offset_to_signer_retrieval_epoch().unwrap())
            .map(|s| s.clone())
            .unwrap_or_else(|| Vec::new());
        certificate_pending.next_signers = store
            .get(&epoch.offset_to_next_signer_retrieval_epoch().unwrap())
            .map(|s| s.clone())
            .unwrap_or_else(|| Vec::new());

        Ok(Some(certificate_pending))
    }

    /// Registers signer with the aggregator
    async fn register_signer(&self, signer: &Signer) -> Result<(), CertificateHandlerError> {
        let epoch = self.get_epoch().await?.offset_to_recording_epoch().unwrap();

        let mut store = self.registered_signers.write().await;
        let mut signers = store
            .get(&epoch)
            .map(|s| s.clone())
            .unwrap_or_else(|| Vec::new());
        signers.push(signer.clone());
        let _ = store.insert(epoch, signers);

        Ok(())
    }

    /// Registers single signatures with the aggregator
    async fn register_signatures(
        &self,
        _signatures: &SingleSignatures,
    ) -> Result<(), CertificateHandlerError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mithril_common::{
        chain_observer::FakeObserver, digesters::DumbImmutableFileObserver, fake_data,
        CardanoNetwork,
    };

    use super::*;

    async fn init() -> (Arc<FakeObserver>, FakeAggregator) {
        let immutable_observer = Arc::new(DumbImmutableFileObserver::new());
        immutable_observer.shall_return(Some(1)).await;
        let chain_observer = Arc::new(FakeObserver::new(Some(Beacon {
            epoch: Epoch(1),
            immutable_file_number: 1,
            network: "devnet".to_string(),
        })));
        let beacon_provider = Arc::new(BeaconProviderImpl::new(
            chain_observer.clone(),
            immutable_observer.clone(),
            CardanoNetwork::DevNet(42),
        ));

        (chain_observer, FakeAggregator::new(beacon_provider))
    }

    #[tokio::test]
    async fn register_signer() {
        let (_, fake_aggregator) = init().await;
        let fake_signers = fake_data::signers(2);
        assert_eq!(2, fake_signers.len());

        fake_aggregator
            .register_signer(&fake_signers.as_slice()[0])
            .await
            .expect("certificate handler should not fail while registering a user");
        let signers = fake_aggregator
            .get_registered_signers(&Epoch(1).offset_to_recording_epoch().unwrap())
            .await
            .expect("we should have a result, None found!");

        assert_eq!(1, signers.len());

        fake_aggregator
            .register_signer(&fake_signers.as_slice()[1])
            .await
            .expect("certificate handler should not fail while registering a user");
        let signers = fake_aggregator
            .get_registered_signers(&Epoch(1).offset_to_recording_epoch().unwrap())
            .await
            .expect("we should have a result, None found!");

        assert_eq!(2, signers.len());
    }

    #[tokio::test]
    async fn retrieve_pending_certificate() {
        let (chain_observer, fake_aggregator) = init().await;
        let cert = fake_aggregator
            .retrieve_pending_certificate()
            .await
            .expect("retrieving a certificate pending should not raise an error");

        assert!(
            cert.is_none(),
            "certificate handler is empty => no pending certificate"
        );

        for signer in fake_data::signers(3) {
            fake_aggregator.register_signer(&signer).await.unwrap();
        }

        let cert = fake_aggregator
            .retrieve_pending_certificate()
            .await
            .expect("retrieving a certificate pending should not raise an error")
            .expect("we should get a pending certificate");

        assert_eq!(0, cert.signers.len());
        assert_eq!(0, cert.next_signers.len());

        chain_observer.next_epoch().await;

        let cert = fake_aggregator
            .retrieve_pending_certificate()
            .await
            .expect("retrieving a certificate pending should not raise an error")
            .expect("we should get a pending certificate");

        assert_eq!(0, cert.signers.len());
        assert_eq!(3, cert.next_signers.len());

        for signer in fake_data::signers(2) {
            fake_aggregator.register_signer(&signer).await.unwrap();
        }

        chain_observer.next_epoch().await;

        let cert = fake_aggregator
            .retrieve_pending_certificate()
            .await
            .expect("retrieving a certificate pending should not raise an error")
            .expect("we should get a pending certificate");

        assert_eq!(3, cert.signers.len());
        assert_eq!(2, cert.next_signers.len());
    }
}
