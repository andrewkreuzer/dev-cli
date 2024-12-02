use log::{info, warn};

use std::path::PathBuf;

use clap::Args;

use crate::clap::Command;
use dev_cli::config::{self, Config};

#[derive(Args)]
pub struct Init {
    #[arg(short, long)]
    global: bool,
}

impl Command for Init {
    async fn run(&self, config: &mut Config) -> Result<(), anyhow::Error> {
        let path = PathBuf::new().join("dev.toml");
        if !path.exists() {
            info!("Creating new config");
            config::create_new(&path)?;
        } else {
            warn!("Local config already exists");
        }

        if self.global {
            config.save_global()?;
        }

        Ok(())
    }
}
