mod aggregator;
mod client;
mod infrastructure;
mod relay_aggregator;
mod relay_passive;
mod relay_signer;
mod signer;

pub use aggregator::{Aggregator, AggregatorConfig};
pub use client::{
    CardanoDbCommand, CardanoDbV2Command, CardanoStakeDistributionCommand,
    CardanoTransactionCommand, Client, ClientCommand, MithrilStakeDistributionCommand,
};
pub use infrastructure::{MithrilInfrastructure, MithrilInfrastructureConfig};
pub use relay_aggregator::RelayAggregator;
pub use relay_passive::RelayPassive;
pub use relay_signer::RelaySigner;
pub use signer::Signer;

pub const DEVNET_MAGIC_ID: mithril_common::MagicId = 42;

pub const GENESIS_VERIFICATION_KEY: &str = "5b33322c3235332c3138362c3230312c3137372c31312c3131372c3133352c3138372c3136372c3138312c3138382c32322c35392c3230362c3130352c3233312c3135302c3231352c33302c37382c3231322c37362c31362c3235322c3138302c37322c3133342c3133372c3234372c3136312c36385d";
pub const GENESIS_SECRET_KEY: &str = "5b3131382c3138342c3232342c3137332c3136302c3234312c36312c3134342c36342c39332c3130362c3232392c38332c3133342c3138392c34302c3138392c3231302c32352c3138342c3136302c3134312c3233372c32362c3136382c35342c3233392c3230342c3133392c3131392c31332c3139395d";
pub const ERA_MARKERS_VERIFICATION_KEY: &str = "5b33322c3235332c3138362c3230312c3137372c31312c3131372c3133352c3138372c3136372c3138312c3138382c32322c35392c3230362c3130352c3233312c3135302c3231352c33302c37382c3231322c37362c31362c3235322c3138302c37322c3133342c3133372c3234372c3136312c36385d";
pub const ERA_MARKERS_SECRET_KEY: &str = "5b3131382c3138342c3232342c3137332c3136302c3234312c36312c3134342c36342c39332c3130362c3232392c38332c3133342c3138392c34302c3138392c3231302c32352c3138342c3136302c3134312c3233372c32362c3136382c35342c3233392c3230342c3133392c3131392c31332c3139395d";
