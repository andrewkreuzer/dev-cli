use log::{info, error};

use dev_cli::{config::Config, github::client};

pub async fn handle_github_command(_config: Config) -> Result<(), anyhow::Error> {
    match client::open_pr("main", "graph", "WHATTTT", "R_kgDOIgwkiA").await {
        Ok(pull_request) => info!("Opened: {:?}", pull_request),
        Err(err) => error!("{err}"),
    }

    Ok(())
}
