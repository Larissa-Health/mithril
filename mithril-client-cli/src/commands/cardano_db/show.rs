use anyhow::{anyhow, Context};
use clap::Parser;
use cli_table::{print_stdout, Cell, Table};

use crate::{
    commands::{client_builder_with_fallback_genesis_key, SharedArgs},
    utils::ExpanderUtils,
    CommandContext,
};
use mithril_client::MithrilResult;

/// Clap command to show a given cardano db
#[derive(Parser, Debug, Clone)]
pub struct CardanoDbShowCommand {
    #[clap(flatten)]
    shared_args: SharedArgs,

    /// Cardano DB digest.
    ///
    /// If `latest` is specified as digest, the command will return the latest cardano db.
    digest: String,
}

impl CardanoDbShowCommand {
    /// Is JSON output enabled
    pub fn is_json_output_enabled(&self) -> bool {
        self.shared_args.json
    }

    /// Cardano DB Show command
    pub async fn execute(&self, context: CommandContext) -> MithrilResult<()> {
        let params = context.config_parameters()?;
        let client = client_builder_with_fallback_genesis_key(&params)?
            .with_logger(context.logger().clone())
            .build()?;

        let get_list_of_artifact_ids = || async {
            let cardano_dbs = client.snapshot().list().await.with_context(|| {
                "Can not get the list of artifacts while retrieving the latest cardano db digest"
            })?;

            Ok(cardano_dbs
                .iter()
                .map(|cardano_db| cardano_db.digest.to_owned())
                .collect::<Vec<String>>())
        };

        let cardano_db_message = client
            .snapshot()
            .get(
                &ExpanderUtils::expand_eventual_id_alias(&self.digest, get_list_of_artifact_ids())
                    .await?,
            )
            .await?
            .ok_or_else(|| anyhow!("Cardano DB not found for digest: '{}'", &self.digest))?;

        if self.is_json_output_enabled() {
            println!("{}", serde_json::to_string(&cardano_db_message)?);
        } else {
            let cardano_db_table = vec![
                vec![
                    "Epoch".cell(),
                    format!("{}", &cardano_db_message.beacon.epoch).cell(),
                ],
                vec![
                    "Immutable File Number".cell(),
                    format!("{}", &cardano_db_message.beacon.immutable_file_number).cell(),
                ],
                vec!["Network".cell(), cardano_db_message.network.cell()],
                vec!["Digest".cell(), cardano_db_message.digest.cell()],
                vec![
                    "Size".cell(),
                    format!("{}", &cardano_db_message.size).cell(),
                ],
                vec![
                    "Cardano node version".cell(),
                    cardano_db_message
                        .cardano_node_version
                        .as_ref()
                        .unwrap_or(&"NA".to_string())
                        .to_string()
                        .cell(),
                ],
                vec![
                    "Location".cell(),
                    cardano_db_message.locations.join(",").cell(),
                ],
                vec![
                    "Created".cell(),
                    cardano_db_message.created_at.to_string().cell(),
                ],
                vec![
                    "Compression Algorithm".cell(),
                    format!(
                        "{}",
                        &cardano_db_message.compression_algorithm.unwrap_or_default()
                    )
                    .cell(),
                ],
            ]
            .table();

            print_stdout(cardano_db_table)?
        }

        Ok(())
    }
}
