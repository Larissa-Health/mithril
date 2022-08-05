use crate::digesters::{ImmutableDigester, ImmutableDigesterError, ImmutableFile};
use crate::entities::ImmutableFileNumber;

use async_trait::async_trait;
use sha2::{Digest, Sha256};
use slog::{debug, info, Logger};
use std::fs::File;
use std::io;
use std::path::PathBuf;

/// A digester working directly on a Cardano DB immutables files
pub struct CardanoImmutableDigester {
    /// A cardano node DB directory
    db_directory: PathBuf,

    /// The logger where the logs should be written
    logger: Logger,
}

impl CardanoImmutableDigester {
    /// ImmutableDigester factory
    pub fn new(db_directory: PathBuf, logger: Logger) -> Self {
        Self {
            db_directory,
            logger,
        }
    }

    fn compute_hash(&self, entries: &[ImmutableFile]) -> Result<[u8; 32], io::Error> {
        let mut hasher = Sha256::new();
        let mut progress = Progress {
            index: 0,
            total: entries.len(),
        };

        for (ix, entry) in entries.iter().enumerate() {
            let mut file = File::open(&entry.path)?;

            io::copy(&mut file, &mut hasher)?;

            if progress.report(ix) {
                info!(self.logger, "hashing: {}", &progress);
            }
        }

        Ok(hasher.finalize().into())
    }
}

#[async_trait]
impl ImmutableDigester for CardanoImmutableDigester {
    async fn compute_digest(
        &self,
        up_to_file_number: ImmutableFileNumber,
    ) -> Result<String, ImmutableDigesterError> {
        let immutables = ImmutableFile::list_completed_in_dir(&*self.db_directory)?
            .into_iter()
            .filter(|f| f.number <= up_to_file_number)
            .collect::<Vec<_>>();

        match immutables.last() {
            None => Err(ImmutableDigesterError::NotEnoughImmutable {
                expected_number: up_to_file_number,
                found_number: None,
            }),
            Some(last_immutable_file) if last_immutable_file.number < up_to_file_number => {
                Err(ImmutableDigesterError::NotEnoughImmutable {
                    expected_number: up_to_file_number,
                    found_number: Some(last_immutable_file.number),
                })
            }
            Some(_) => {
                info!(self.logger, "#immutables: {}", immutables.len());

                let hash = self
                    .compute_hash(&immutables)
                    .map_err(ImmutableDigesterError::DigestComputationError)?;
                let digest = hex::encode(hash);

                debug!(self.logger, "#computed digest: {:?}", digest);

                Ok(digest)
            }
        }
    }
}

struct Progress {
    index: usize,
    total: usize,
}

impl Progress {
    fn report(&mut self, ix: usize) -> bool {
        self.index = ix;
        (20 * ix) % self.total == 0
    }

    fn percent(&self) -> f64 {
        (self.index as f64 * 100.0 / self.total as f64).ceil()
    }
}

impl std::fmt::Display for Progress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}/{} ({}%)", self.index, self.total, self.percent())
    }
}

#[cfg(test)]
mod tests {
    use super::Progress;

    #[test]
    fn reports_progress_every_5_percent() {
        let mut progress = Progress {
            index: 0,
            total: 7000,
        };

        assert!(!progress.report(1));
        assert!(!progress.report(4));
        assert!(progress.report(350));
        assert!(!progress.report(351));
    }

    #[test]
    fn reports_progress_when_total_lower_than_20() {
        let mut progress = Progress {
            index: 0,
            total: 16,
        };

        assert!(progress.report(4));
        assert!(progress.report(12));
        assert!(!progress.report(3));
        assert!(!progress.report(15));
    }
}