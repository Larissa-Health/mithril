use std::sync::Arc;

use async_trait::async_trait;
use slog::{debug, Logger};
use tokio::sync::Mutex;

use crate::cardano_block_scanner::BlockStreamer;
use crate::cardano_block_scanner::ChainScannedBlocks;
use crate::chain_reader::{ChainBlockNextAction, ChainBlockReader};
use crate::entities::BlockNumber;
use crate::entities::ChainPoint;
use crate::StdResult;

/// The action that indicates what to do next with the streamer
#[derive(Debug, Clone, PartialEq)]
enum BlockStreamerNextAction {
    /// Use a [ChainBlockNextAction]
    ChainBlockNextAction(ChainBlockNextAction),
    /// Skip to the next action
    SkipToNextAction,
}

/// The maximum number of roll forwards during a poll
const MAX_ROLL_FORWARDS_PER_POLL: usize = 100;

/// [Block streamer][BlockStreamer] that streams blocks with a [Chain block reader][ChainBlockReader]
pub struct ChainReaderBlockStreamer {
    chain_reader: Arc<Mutex<dyn ChainBlockReader>>,
    from: ChainPoint,
    until: BlockNumber,
    max_roll_forwards_per_poll: usize,
    logger: Logger,
}

#[async_trait]
impl BlockStreamer for ChainReaderBlockStreamer {
    async fn poll_next(&mut self) -> StdResult<Option<ChainScannedBlocks>> {
        debug!(self.logger, "ChainReaderBlockStreamer polls next");

        let chain_scanned_blocks: ChainScannedBlocks;
        let mut roll_forwards = vec![];
        loop {
            let block_streamer_next_action = self.get_next_chain_block_action().await?;
            match block_streamer_next_action {
                Some(BlockStreamerNextAction::ChainBlockNextAction(
                    ChainBlockNextAction::RollForward {
                        next_point: _,
                        parsed_block,
                    },
                )) => {
                    roll_forwards.push(parsed_block);
                    if roll_forwards.len() >= self.max_roll_forwards_per_poll {
                        return Ok(Some(ChainScannedBlocks::RollForwards(roll_forwards)));
                    }
                }
                Some(BlockStreamerNextAction::ChainBlockNextAction(
                    ChainBlockNextAction::RollBackward { rollback_point },
                )) => {
                    if roll_forwards.is_empty() {
                        chain_scanned_blocks = ChainScannedBlocks::RollBackward(rollback_point);
                        return Ok(Some(chain_scanned_blocks));
                    } else {
                        chain_scanned_blocks = ChainScannedBlocks::RollForwards(roll_forwards);
                        return Ok(Some(chain_scanned_blocks));
                    }
                }
                Some(BlockStreamerNextAction::SkipToNextAction) => {
                    continue;
                }
                None => {
                    if roll_forwards.is_empty() {
                        return Ok(None);
                    } else {
                        chain_scanned_blocks = ChainScannedBlocks::RollForwards(roll_forwards);
                        return Ok(Some(chain_scanned_blocks));
                    }
                }
            }
        }
    }
}

impl ChainReaderBlockStreamer {
    /// Factory
    pub async fn try_new(
        chain_reader: Arc<Mutex<dyn ChainBlockReader>>,
        from: Option<ChainPoint>,
        until: BlockNumber,
        logger: Logger,
    ) -> StdResult<Self> {
        let from = from.unwrap_or(ChainPoint::origin());
        {
            let mut chain_reader_inner = chain_reader.try_lock()?;
            chain_reader_inner.set_chain_point(&from).await?;
        }
        Ok(Self {
            chain_reader,
            from,
            until,
            max_roll_forwards_per_poll: MAX_ROLL_FORWARDS_PER_POLL,
            logger,
        })
    }

    async fn get_next_chain_block_action(&self) -> StdResult<Option<BlockStreamerNextAction>> {
        let mut chain_reader = self.chain_reader.try_lock()?;
        match chain_reader.get_next_chain_block().await? {
            Some(ChainBlockNextAction::RollForward {
                next_point,
                parsed_block,
            }) => {
                debug!(self.logger, "RollForward ({next_point:?})");
                if next_point.block_number >= self.until {
                    debug!(
                        self.logger,
                        "ChainReaderBlockStreamer received a RollForward({next_point:?}) above threshold block number ({})",
                        next_point.block_number
                    );
                    Ok(None)
                } else {
                    debug!(
                        self.logger,
                        "ChainReaderBlockStreamer received a RollForward({next_point:?}) below threshold block number ({})",
                        next_point.block_number
                    );
                    chain_reader.set_chain_point(&next_point).await?;
                    Ok(Some(BlockStreamerNextAction::ChainBlockNextAction(
                        ChainBlockNextAction::RollForward {
                            next_point,
                            parsed_block,
                        },
                    )))
                }
            }
            Some(ChainBlockNextAction::RollBackward { rollback_point }) => {
                debug!(
                    self.logger,
                    "ChainReaderBlockStreamer received a RollBackward({rollback_point:?})"
                );
                let block_streamer_next_action = if rollback_point == self.from {
                    BlockStreamerNextAction::SkipToNextAction
                } else {
                    chain_reader.set_chain_point(&rollback_point).await?;
                    BlockStreamerNextAction::ChainBlockNextAction(
                        ChainBlockNextAction::RollBackward { rollback_point },
                    )
                };
                Ok(Some(block_streamer_next_action))
            }
            None => {
                debug!(self.logger, "ChainReaderBlockStreamer received nothing");
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cardano_block_scanner::ScannedBlock;
    use crate::chain_reader::FakeChainReader;
    use crate::test_utils::TestLogger;

    use super::*;

    #[tokio::test]
    async fn test_parse_expected_nothing_above_block_number_threshold() {
        let logger = TestLogger::stdout();
        let chain_reader = Arc::new(Mutex::new(FakeChainReader::new(vec![
            ChainBlockNextAction::RollForward {
                next_point: ChainPoint::new(100, 10, "hash-123"),
                parsed_block: ScannedBlock::new("hash-1", 1, 10, 20, Vec::<&str>::new()),
            },
        ])));
        let mut block_streamer =
            ChainReaderBlockStreamer::try_new(chain_reader, None, 1, logger.clone())
                .await
                .unwrap();

        let scanned_blocks = block_streamer.poll_next().await.expect("poll_next failed");

        assert_eq!(None, scanned_blocks,);
    }

    #[tokio::test]
    async fn test_parse_expected_multiple_rollforwards_below_block_number_threshold() {
        let logger = TestLogger::stdout();
        let chain_reader = Arc::new(Mutex::new(FakeChainReader::new(vec![
            ChainBlockNextAction::RollForward {
                next_point: ChainPoint::new(100, 10, "hash-123"),
                parsed_block: ScannedBlock::new("hash-1", 1, 10, 1, Vec::<&str>::new()),
            },
            ChainBlockNextAction::RollForward {
                next_point: ChainPoint::new(200, 20, "hash-456"),
                parsed_block: ScannedBlock::new("hash-2", 2, 20, 1, Vec::<&str>::new()),
            },
        ])));
        let mut block_streamer =
            ChainReaderBlockStreamer::try_new(chain_reader, None, 100, logger.clone())
                .await
                .unwrap();

        let scanned_blocks = block_streamer.poll_next().await.expect("poll_next failed");

        assert_eq!(
            Some(ChainScannedBlocks::RollForwards(vec![
                ScannedBlock::new("hash-1", 1, 10, 1, Vec::<&str>::new()),
                ScannedBlock::new("hash-2", 2, 20, 1, Vec::<&str>::new())
            ])),
            scanned_blocks,
        );
    }

    #[tokio::test]
    async fn test_parse_expected_maximum_rollforwards_retrieved_per_poll() {
        let logger = TestLogger::stdout();
        let chain_reader = Arc::new(Mutex::new(FakeChainReader::new(vec![
            ChainBlockNextAction::RollForward {
                next_point: ChainPoint::new(100, 10, "hash-123"),
                parsed_block: ScannedBlock::new("hash-1", 1, 10, 1, Vec::<&str>::new()),
            },
            ChainBlockNextAction::RollForward {
                next_point: ChainPoint::new(200, 20, "hash-456"),
                parsed_block: ScannedBlock::new("hash-2", 2, 20, 1, Vec::<&str>::new()),
            },
            ChainBlockNextAction::RollForward {
                next_point: ChainPoint::new(300, 30, "hash-789"),
                parsed_block: ScannedBlock::new("hash-3", 3, 30, 1, Vec::<&str>::new()),
            },
        ])));
        let mut block_streamer =
            ChainReaderBlockStreamer::try_new(chain_reader, None, 100, logger.clone())
                .await
                .unwrap();
        block_streamer.max_roll_forwards_per_poll = 2;

        let scanned_blocks = block_streamer.poll_next().await.expect("poll_next failed");

        assert_eq!(
            Some(ChainScannedBlocks::RollForwards(vec![
                ScannedBlock::new("hash-1", 1, 10, 1, Vec::<&str>::new()),
                ScannedBlock::new("hash-2", 2, 20, 1, Vec::<&str>::new())
            ])),
            scanned_blocks,
        );
    }

    #[tokio::test]
    async fn test_parse_expected_nothing_when_rollbackward_on_same_point() {
        let logger = TestLogger::stdout();
        let chain_reader = Arc::new(Mutex::new(FakeChainReader::new(vec![
            ChainBlockNextAction::RollBackward {
                rollback_point: ChainPoint::new(100, 10, "hash-123"),
            },
        ])));
        let mut block_streamer = ChainReaderBlockStreamer::try_new(
            chain_reader,
            Some(ChainPoint::new(100, 10, "hash-123")),
            1,
            logger.clone(),
        )
        .await
        .unwrap();

        let scanned_blocks = block_streamer.poll_next().await.expect("poll_next failed");

        assert_eq!(None, scanned_blocks,);
    }

    #[tokio::test]
    async fn test_parse_expected_rollbackward_when_on_different_point_and_no_previous_rollforward()
    {
        let logger = TestLogger::stdout();
        let chain_reader = Arc::new(Mutex::new(FakeChainReader::new(vec![
            ChainBlockNextAction::RollBackward {
                rollback_point: ChainPoint::new(100, 10, "hash-123"),
            },
        ])));
        let mut block_streamer =
            ChainReaderBlockStreamer::try_new(chain_reader, None, 1, logger.clone())
                .await
                .unwrap();

        let scanned_blocks = block_streamer.poll_next().await.expect("poll_next failed");

        assert_eq!(
            Some(ChainScannedBlocks::RollBackward(ChainPoint::new(
                100, 10, "hash-123"
            ))),
            scanned_blocks,
        );
    }

    #[tokio::test]
    async fn test_parse_expected_rollforward_when_rollbackward_on_different_point_and_have_previous_rollforwards(
    ) {
        let logger = TestLogger::stdout();
        let chain_reader = Arc::new(Mutex::new(FakeChainReader::new(vec![
            ChainBlockNextAction::RollForward {
                next_point: ChainPoint::new(80, 8, "hash-888"),
                parsed_block: ScannedBlock::new("hash-8", 80, 8, 1, Vec::<&str>::new()),
            },
            ChainBlockNextAction::RollForward {
                next_point: ChainPoint::new(90, 9, "hash-999"),
                parsed_block: ScannedBlock::new("hash-9", 90, 9, 1, Vec::<&str>::new()),
            },
            ChainBlockNextAction::RollBackward {
                rollback_point: ChainPoint::new(100, 10, "hash-123"),
            },
        ])));
        let mut block_streamer =
            ChainReaderBlockStreamer::try_new(chain_reader, None, 1000, logger.clone())
                .await
                .unwrap();

        let scanned_blocks = block_streamer.poll_next().await.expect("poll_next failed");

        assert_eq!(
            Some(ChainScannedBlocks::RollForwards(vec![
                ScannedBlock::new("hash-8", 80, 8, 1, Vec::<&str>::new()),
                ScannedBlock::new("hash-9", 90, 9, 1, Vec::<&str>::new())
            ])),
            scanned_blocks,
        );
    }

    #[tokio::test]
    async fn test_parse_expected_nothing() {
        let logger = TestLogger::stdout();
        let chain_reader = Arc::new(Mutex::new(FakeChainReader::new(vec![])));
        let mut block_streamer =
            ChainReaderBlockStreamer::try_new(chain_reader, None, 1, logger.clone())
                .await
                .unwrap();

        let scanned_blocks = block_streamer.poll_next().await.expect("poll_next failed");

        assert_eq!(scanned_blocks, None);
    }
}
