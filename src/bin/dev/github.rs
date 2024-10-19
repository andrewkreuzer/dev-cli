use log::error;
use clap::Args;
#[cfg(feature = "github")]
use log::info;

use dev_cli::config::Config;
use crate::clap::Command;
#[cfg(feature = "github")]
use dev_cli::github::client;

#[derive(Args)]
pub struct Github {}

impl Command for Github {
    async fn run(&self, _config: &mut Config) -> Result<(), anyhow::Error> {
        #[cfg(not(feature = "github"))] error!("Github feature is not enabled");
        #[cfg(feature = "github")]
        match client::open_pr("main", "graph", "WHATTTT", "R_kgDOIgwkiA").await {
            Ok(pull_request) => info!("Opened: {:?}", pull_request),
            Err(err) => error!("{err}"),
        }
        Ok(())
    }
}
