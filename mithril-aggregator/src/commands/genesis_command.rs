use anyhow::Context;
use clap::{Parser, Subcommand};
use config::{builder::DefaultState, ConfigBuilder};
use mithril_common::{
    crypto_helper::{ProtocolGenesisSecretKey, ProtocolGenesisSigner},
    entities::HexEncodedGenesisSecretKey,
    StdResult,
};
use slog::{debug, Logger};
use std::{path::PathBuf, sync::Arc};

use crate::{
    dependency_injection::DependenciesBuilder, tools::GenesisTools, ServeCommandConfiguration,
};

/// Genesis tools
#[derive(Parser, Debug, Clone)]
pub struct GenesisCommand {
    /// commands
    #[clap(subcommand)]
    pub genesis_subcommand: GenesisSubCommand,
}

impl GenesisCommand {
    pub async fn execute(
        &self,
        root_logger: Logger,
        config_builder: ConfigBuilder<DefaultState>,
    ) -> StdResult<()> {
        self.genesis_subcommand
            .execute(root_logger, config_builder)
            .await
    }
}

/// Genesis tools commands.
#[derive(Debug, Clone, Subcommand)]
pub enum GenesisSubCommand {
    /// Genesis certificate export command.
    Export(ExportGenesisSubCommand),

    /// Genesis certificate import command.
    Import(ImportGenesisSubCommand),

    /// Genesis certificate sign command.
    Sign(SignGenesisSubCommand),

    /// Genesis certificate bootstrap command.
    Bootstrap(BootstrapGenesisSubCommand),

    /// Genesis keypair generation command.
    GenerateKeypair(GenerateKeypairGenesisSubCommand),
}

impl GenesisSubCommand {
    pub async fn execute(
        &self,
        root_logger: Logger,
        config_builder: ConfigBuilder<DefaultState>,
    ) -> StdResult<()> {
        match self {
            Self::Bootstrap(cmd) => cmd.execute(root_logger, config_builder).await,
            Self::Export(cmd) => cmd.execute(root_logger, config_builder).await,
            Self::Import(cmd) => cmd.execute(root_logger, config_builder).await,
            Self::Sign(cmd) => cmd.execute(root_logger, config_builder).await,
            Self::GenerateKeypair(cmd) => cmd.execute(root_logger, config_builder).await,
        }
    }
}

/// Genesis certificate export command
#[derive(Parser, Debug, Clone)]
pub struct ExportGenesisSubCommand {
    /// Target path
    #[clap(long)]
    target_path: PathBuf,
}

impl ExportGenesisSubCommand {
    pub async fn execute(
        &self,
        root_logger: Logger,
        config_builder: ConfigBuilder<DefaultState>,
    ) -> StdResult<()> {
        let mut config: ServeCommandConfiguration = config_builder
            .build()
            .with_context(|| "configuration build error")?
            .try_deserialize()
            .with_context(|| "configuration deserialize error")?;
        // TODO: `store_retention_limit` will be set in the specific configuration implementation of the genesis command.
        config.store_retention_limit = None;
        debug!(root_logger, "EXPORT GENESIS command"; "config" => format!("{config:?}"));
        println!(
            "Genesis export payload to sign to {}",
            self.target_path.display()
        );
        let mut dependencies_builder =
            DependenciesBuilder::new(root_logger.clone(), Arc::new(config.clone()));
        let dependencies = dependencies_builder
            .create_genesis_container()
            .await
            .with_context(|| {
                "Dependencies Builder can not create genesis command dependencies container"
            })?;

        let genesis_tools = GenesisTools::from_dependencies(dependencies)
            .await
            .with_context(|| "genesis-tools: initialization error")?;
        genesis_tools
            .export_payload_to_sign(&self.target_path)
            .with_context(|| "genesis-tools: export error")?;
        Ok(())
    }
}

#[derive(Parser, Debug, Clone)]
pub struct ImportGenesisSubCommand {
    /// Signed Payload Path
    #[clap(long)]
    signed_payload_path: PathBuf,
}

impl ImportGenesisSubCommand {
    pub async fn execute(
        &self,
        root_logger: Logger,
        config_builder: ConfigBuilder<DefaultState>,
    ) -> StdResult<()> {
        let mut config: ServeCommandConfiguration = config_builder
            .build()
            .with_context(|| "configuration build error")?
            .try_deserialize()
            .with_context(|| "configuration deserialize error")?;
        // TODO: `store_retention_limit` will be set in the specific configuration implementation of the genesis command.
        config.store_retention_limit = None;
        debug!(root_logger, "IMPORT GENESIS command"; "config" => format!("{config:?}"));
        println!(
            "Genesis import signed payload from {}",
            self.signed_payload_path.to_string_lossy()
        );
        let mut dependencies_builder =
            DependenciesBuilder::new(root_logger.clone(), Arc::new(config.clone()));
        let dependencies = dependencies_builder
            .create_genesis_container()
            .await
            .with_context(|| {
                "Dependencies Builder can not create genesis command dependencies container"
            })?;

        let genesis_tools = GenesisTools::from_dependencies(dependencies)
            .await
            .with_context(|| "genesis-tools: initialization error")?;
        genesis_tools
            .import_payload_signature(&self.signed_payload_path)
            .await
            .with_context(|| "genesis-tools: import error")?;
        Ok(())
    }
}

#[derive(Parser, Debug, Clone)]
pub struct SignGenesisSubCommand {
    /// To Sign Payload Path
    #[clap(long)]
    to_sign_payload_path: PathBuf,

    /// Target Signed Payload Path
    #[clap(long)]
    target_signed_payload_path: PathBuf,

    /// Genesis Secret Key Path
    #[clap(long)]
    genesis_secret_key_path: PathBuf,
}

impl SignGenesisSubCommand {
    pub async fn execute(
        &self,
        root_logger: Logger,
        _config_builder: ConfigBuilder<DefaultState>,
    ) -> StdResult<()> {
        debug!(root_logger, "SIGN GENESIS command");
        println!(
            "Genesis sign payload from {} to {}",
            self.to_sign_payload_path.to_string_lossy(),
            self.target_signed_payload_path.to_string_lossy()
        );

        GenesisTools::sign_genesis_certificate(
            &self.to_sign_payload_path,
            &self.target_signed_payload_path,
            &self.genesis_secret_key_path,
        )
        .await
        .with_context(|| "genesis-tools: sign error")?;

        Ok(())
    }
}
#[derive(Parser, Debug, Clone)]
pub struct BootstrapGenesisSubCommand {
    /// Genesis Secret Key (test only)
    #[clap(long, env = "GENESIS_SECRET_KEY")]
    genesis_secret_key: HexEncodedGenesisSecretKey,
}

impl BootstrapGenesisSubCommand {
    pub async fn execute(
        &self,
        root_logger: Logger,
        config_builder: ConfigBuilder<DefaultState>,
    ) -> StdResult<()> {
        let mut config: ServeCommandConfiguration = config_builder
            .build()
            .with_context(|| "configuration build error")?
            .try_deserialize()
            .with_context(|| "configuration deserialize error")?;
        // TODO: `store_retention_limit` will be set in the specific configuration implementation of the genesis command.
        config.store_retention_limit = None;
        debug!(root_logger, "BOOTSTRAP GENESIS command"; "config" => format!("{config:?}"));
        println!("Genesis bootstrap for test only!");
        let mut dependencies_builder =
            DependenciesBuilder::new(root_logger.clone(), Arc::new(config.clone()));
        let dependencies = dependencies_builder
            .create_genesis_container()
            .await
            .with_context(|| {
                "Dependencies Builder can not create genesis command dependencies container"
            })?;

        let genesis_tools = GenesisTools::from_dependencies(dependencies)
            .await
            .with_context(|| "genesis-tools: initialization error")?;
        let genesis_secret_key = ProtocolGenesisSecretKey::from_json_hex(&self.genesis_secret_key)
            .with_context(|| "json hex decode of genesis secret key failure")?;
        let genesis_signer = ProtocolGenesisSigner::from_secret_key(genesis_secret_key);
        genesis_tools
            .bootstrap_test_genesis_certificate(genesis_signer)
            .await
            .with_context(|| "genesis-tools: bootstrap error")?;
        Ok(())
    }
}

/// Genesis keypair generation command.
#[derive(Parser, Debug, Clone)]
pub struct GenerateKeypairGenesisSubCommand {
    /// Target path for the generated keypair
    #[clap(long)]
    target_path: PathBuf,
}

impl GenerateKeypairGenesisSubCommand {
    pub async fn execute(
        &self,
        root_logger: Logger,
        _config_builder: ConfigBuilder<DefaultState>,
    ) -> StdResult<()> {
        debug!(root_logger, "GENERATE KEYPAIR GENESIS command");
        println!(
            "Genesis generate keypair to {}",
            self.target_path.to_string_lossy()
        );

        GenesisTools::create_and_save_genesis_keypair(&self.target_path)
            .with_context(|| "genesis-tools: keypair generation error")?;

        Ok(())
    }
}
