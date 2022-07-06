use std::path::PathBuf;

use crate::snapshot_uploaders::SnapshotLocation;
use crate::{DependencyManager, SnapshotError, Snapshotter};
use async_trait::async_trait;
use chrono::Utc;
use mithril_common::digesters::{Digester, DigesterResult, ImmutableDigester};
use mithril_common::entities::{
    Beacon, Certificate, CertificatePending, SignerWithStake, Snapshot,
};
use mithril_common::{store::stake_store::StakeStorer, CardanoNetwork};

use slog_scope::{debug, error, info, trace, warn};
use std::path::Path;
use std::sync::Arc;

#[cfg(test)]
use mockall::automock;

use super::RuntimeError;
pub struct AggregatorConfig {
    /// Interval between each snapshot, in ms
    pub interval: u64,

    /// Cardano network
    pub network: CardanoNetwork,

    /// DB directory to snapshot
    pub db_directory: PathBuf,

    /// Directory to store snapshot
    pub snapshot_directory: PathBuf,

    /// Services dependencies
    pub dependencies: Arc<DependencyManager>,
}

impl AggregatorConfig {
    pub fn new(
        interval: u64,
        network: CardanoNetwork,
        db_directory: &Path,
        snapshot_directory: &Path,
        dependencies: Arc<DependencyManager>,
    ) -> Self {
        Self {
            interval,
            network,
            db_directory: db_directory.to_path_buf(),
            snapshot_directory: snapshot_directory.to_path_buf(),
            dependencies,
        }
    }
}

#[async_trait]
pub trait AggregatorRunnerTrait: Sync + Send {
    /// Return the current beacon if it is newer than the given one.
    async fn is_new_beacon(&self, beacon: Option<Beacon>) -> Result<Option<Beacon>, RuntimeError>;

    async fn compute_digest(&self, new_beacon: &Beacon) -> Result<DigesterResult, RuntimeError>;

    async fn update_beacon(&self, new_beacon: &Beacon) -> Result<(), RuntimeError>;

    async fn update_stake_distribution(&self, new_beacon: &Beacon) -> Result<(), RuntimeError>;

    async fn update_message_in_multisigner(
        &self,
        digest_result: DigesterResult,
    ) -> Result<(), RuntimeError>;

    async fn create_new_pending_certificate_from_multisigner(
        &self,
        beacon: Beacon,
    ) -> Result<CertificatePending, RuntimeError>;

    async fn save_pending_certificate(
        &self,
        pending_certificate: CertificatePending,
    ) -> Result<(), RuntimeError>;

    async fn drop_pending_certificate(&self) -> Result<CertificatePending, RuntimeError>;

    async fn is_multisig_created(&self) -> Result<bool, RuntimeError>;

    async fn create_snapshot_archive(&self) -> Result<PathBuf, RuntimeError>;

    async fn upload_snapshot_archive(
        &self,
        path: &Path,
    ) -> Result<Vec<SnapshotLocation>, RuntimeError>;

    async fn create_and_save_certificate(
        &self,
        beacon: &Beacon,
        certificate_pending: &CertificatePending,
    ) -> Result<Certificate, RuntimeError>;

    async fn create_and_save_snapshot(
        &self,
        certificate: Certificate,
        file_path: &Path,
        remote_locations: Vec<String>,
    ) -> Result<Snapshot, RuntimeError>;
}

pub struct AggregatorRunner {
    config: AggregatorConfig,
}

impl AggregatorRunner {
    pub fn new(config: AggregatorConfig) -> Self {
        Self { config }
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
impl AggregatorRunnerTrait for AggregatorRunner {
    /// Is there a new beacon?
    /// returns a new beacon if there is one more recent than the given one
    async fn is_new_beacon(
        &self,
        maybe_beacon: Option<Beacon>,
    ) -> Result<Option<Beacon>, RuntimeError> {
        info!("checking if there is a new beacon");
        let current_beacon = self
            .config
            .dependencies
            .beacon_provider
            .as_ref()
            .ok_or_else(|| {
                RuntimeError::General("no beacon provider registered".to_string().into())
            })?
            .read()
            .await
            .get_current_beacon()
            .await
            .map_err(RuntimeError::General)?;

        debug!("checking if there is a new beacon: {:?}", current_beacon);

        match maybe_beacon {
            Some(beacon) if current_beacon > beacon => Ok(Some(current_beacon)),
            None => Ok(Some(current_beacon)),
            _ => Ok(None),
        }
    }

    /// Is a multisignature ready?
    /// Can we create a multisignature.
    async fn is_multisig_created(&self) -> Result<bool, RuntimeError> {
        info!("check if we can create a multisignature");
        let has_multisig = self
            .config
            .dependencies
            .multi_signer
            .as_ref()
            .ok_or_else(|| RuntimeError::General("no multisigner registered".to_string().into()))?
            .write()
            .await
            .create_multi_signature()
            .await?
            .is_some();

        if has_multisig {
            debug!("new MULTISIG created");
        } else {
            info!("no multisig created");
        }
        Ok(has_multisig)
    }

    async fn compute_digest(&self, new_beacon: &Beacon) -> Result<DigesterResult, RuntimeError> {
        info!("running runner::compute_digester");
        let digester =
            ImmutableDigester::new(self.config.db_directory.clone(), slog_scope::logger());
        debug!("computing digest"; "db_directory" => self.config.db_directory.display());

        // digest is done in a separate thread because it is blocking the whole task
        debug!("launching digester thread");
        let digest_result = tokio::task::spawn_blocking(move || digester.compute_digest())
            .await
            .map_err(|e| RuntimeError::General(e.into()))??;
        debug!(
            "last immutable file number: {}",
            digest_result.last_immutable_file_number
        );

        if digest_result.last_immutable_file_number != new_beacon.immutable_file_number {
            error!("digest beacon is different than the given beacon");
            Err(RuntimeError::General(
                format!("The digest has been computed for a different immutable ({}) file than the one given in the beacon ({}).", digest_result.last_immutable_file_number, new_beacon.immutable_file_number).into()
            ))
        } else {
            trace!("digest last immutable file number and new beacon file number are consistent");
            Ok(digest_result)
        }
    }

    async fn update_beacon(&self, new_beacon: &Beacon) -> Result<(), RuntimeError> {
        info!("update beacon"; "beacon" => #?new_beacon);
        let _ = self
            .config
            .dependencies
            .beacon_store
            .as_ref()
            .ok_or_else(|| RuntimeError::General("no beacon store registered".to_string().into()))?
            .write()
            .await
            .set_current_beacon(new_beacon.to_owned())
            .await?;
        Ok(())
    }

    async fn update_stake_distribution(&self, new_beacon: &Beacon) -> Result<(), RuntimeError> {
        info!("update stake distribution"; "beacon" => #?new_beacon);
        let stake_distribution = self
            .config
            .dependencies
            .chain_observer
            .as_ref()
            .ok_or_else(|| {
                RuntimeError::General("no chain observer registered".to_string().into())
            })?
            .read()
            .await
            .get_current_stake_distribution()
            .await?
            .ok_or_else(|| RuntimeError::General("no epoch was returned".to_string().into()))?;
        let mut stake_store = self
            .config
            .dependencies
            .stake_store
            .as_ref()
            .ok_or_else(|| RuntimeError::General("no stake store registered".to_string().into()))?
            .write()
            .await;

        for (party_id, stake) in &stake_distribution {
            stake_store
                .save_stake(
                    new_beacon.epoch,
                    SignerWithStake::new(party_id.to_owned(), "".to_string(), *stake),
                )
                .await?;
        }
        Ok(())
    }

    async fn create_new_pending_certificate_from_multisigner(
        &self,
        beacon: Beacon,
    ) -> Result<CertificatePending, RuntimeError> {
        info!("running runner::create_pending_certificate");
        let multi_signer = self
            .config
            .dependencies
            .multi_signer
            .as_ref()
            .ok_or_else(|| RuntimeError::General("no multisigner registered".to_string().into()))?
            .read()
            .await;

        debug!("creating certificate pending using multisigner");
        warn!("pending certificate's previous hash is fake");
        let pending_certificate = CertificatePending::new(
            beacon,
            multi_signer
                .get_protocol_parameters()
                .await
                .ok_or_else(|| RuntimeError::General("no protocol parameters".to_string().into()))?
                .into(),
            "123".to_string(),
            multi_signer.get_signers().await?,
        );

        Ok(pending_certificate)
    }

    async fn save_pending_certificate(
        &self,
        pending_certificate: CertificatePending,
    ) -> Result<(), RuntimeError> {
        info!("saving pending certificate");

        self.config
            .dependencies
            .certificate_pending_store
            .as_ref()
            .ok_or_else(|| {
                RuntimeError::General("no certificate pending store registered".to_string().into())
            })?
            .write()
            .await
            .save(pending_certificate)
            .await
            .map_err(|e| e.into())
    }

    async fn update_message_in_multisigner(
        &self,
        digest_result: DigesterResult,
    ) -> Result<(), RuntimeError> {
        info!("update message in multisigner");

        self.config
            .dependencies
            .multi_signer
            .as_ref()
            .ok_or_else(|| RuntimeError::General("no multisigner registered".to_string().into()))?
            .write()
            .await
            .update_current_message(digest_result.digest)
            .await
            .map_err(RuntimeError::MultiSigner)
    }

    async fn drop_pending_certificate(&self) -> Result<CertificatePending, RuntimeError> {
        info!("drop pending certificate");

        let certificate_pending = self
            .config
            .dependencies
            .certificate_pending_store
            .as_ref()
            .ok_or_else(|| {
                RuntimeError::General("no certificate pending store registered".to_string().into())
            })?
            .write()
            .await
            .remove()
            .await?
            .ok_or_else(|| {
                RuntimeError::General(
                    "no certificate pending for the given beacon"
                        .to_string()
                        .into(),
                )
            })?;

        Ok(certificate_pending)
    }

    async fn create_snapshot_archive(&self) -> Result<PathBuf, RuntimeError> {
        info!("create snapshot archive");

        let snapshotter = Snapshotter::new(
            self.config.db_directory.clone(),
            self.config.snapshot_directory.clone(),
        );
        let message = self
            .config
            .dependencies
            .multi_signer
            .as_ref()
            .ok_or_else(|| RuntimeError::General("no multisigner registered".to_string().into()))?
            .read()
            .await
            .get_current_message()
            .await
            .ok_or_else(|| RuntimeError::General("no message found".to_string().into()))?;
        let snapshot_name = format!("{}.{}.tar.gz", self.config.network, &message);
        // spawn a separate thread to prevent blocking
        let snapshot_path =
            tokio::task::spawn_blocking(move || -> Result<PathBuf, SnapshotError> {
                snapshotter.snapshot(&snapshot_name)
            })
            .await
            .map_err(|e| RuntimeError::General(e.into()))??;

        debug!("snapshot created at '{}'", snapshot_path.to_string_lossy());

        Ok(snapshot_path)
    }

    async fn create_and_save_certificate(
        &self,
        beacon: &Beacon,
        certificate_pending: &CertificatePending,
    ) -> Result<Certificate, RuntimeError> {
        info!("create and save certificate");
        let multisigner = self
            .config
            .dependencies
            .multi_signer
            .as_ref()
            .ok_or_else(|| RuntimeError::General("no multisigner registered".to_string().into()))?
            .read()
            .await;
        let certificate = multisigner
            .create_certificate(beacon.clone(), certificate_pending.previous_hash.clone())
            .await?
            .ok_or_else(|| RuntimeError::General("no certificate generated".to_string().into()))?;
        let _ = self
            .config
            .dependencies
            .certificate_store
            .as_ref()
            .ok_or_else(|| {
                RuntimeError::General("no certificate store registered".to_string().into())
            })?
            .write()
            .await
            .save(certificate.clone())
            .await?;

        Ok(certificate)
    }

    async fn upload_snapshot_archive(
        &self,
        path: &Path,
    ) -> Result<Vec<SnapshotLocation>, RuntimeError> {
        info!("upload snapshot archive");
        let location = self
            .config
            .dependencies
            .snapshot_uploader
            .as_ref()
            .ok_or_else(|| {
                RuntimeError::SnapshotUploader("no snapshot uploader registered".to_string())
            })?
            .read()
            .await
            .upload_snapshot(path)
            .await
            .map_err(RuntimeError::SnapshotUploader)?;

        Ok(vec![location])
    }

    async fn create_and_save_snapshot(
        &self,
        certificate: Certificate,
        file_path: &Path,
        remote_locations: Vec<String>,
    ) -> Result<Snapshot, RuntimeError> {
        let snapshot = Snapshot::new(
            certificate.digest,
            certificate.hash,
            std::fs::metadata(file_path)
                .map_err(|e| RuntimeError::General(e.into()))?
                .len(),
            format!("{:?}", Utc::now()),
            remote_locations,
        );

        let _ = self
            .config
            .dependencies
            .snapshot_store
            .as_ref()
            .ok_or_else(|| {
                RuntimeError::General("no snapshot store registered".to_string().into())
            })?
            .write()
            .await
            .add_snapshot(snapshot.clone())
            .await?;

        Ok(snapshot)
    }
}