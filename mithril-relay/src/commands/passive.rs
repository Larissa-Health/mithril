use clap::Parser;
use libp2p::Multiaddr;
use mithril_common::StdResult;
use slog::error;

use super::CommandContext;
use crate::PassiveRelay;

#[derive(Parser, Debug, Clone)]
pub struct PassiveCommand {
    /// Peer listening port
    #[clap(long, env = "LISTEN_PORT", default_value_t = 0)]
    listen_port: u16,

    /// Dial to peer multi-address (e.g. /ip4/0.0.0.0/tcp/1234)
    #[clap(long, env = "DIAL_TO")]
    dial_to: Option<Multiaddr>,
}

impl PassiveCommand {
    /// Main command execution
    pub async fn execute(&self, context: CommandContext) -> StdResult<()> {
        let dial_to = self.dial_to.to_owned();
        let addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", self.listen_port).parse()?;
        let logger = context.logger();

        let mut relay = PassiveRelay::start(&addr, logger).await?;
        if let Some(dial_to_address) = dial_to {
            relay.dial_peer(dial_to_address.clone())?;
        }
        loop {
            if let Err(err) = relay.tick().await {
                error!(logger, "P2PClient: tick error"; "error" => ?err);
            }
        }
    }
}
