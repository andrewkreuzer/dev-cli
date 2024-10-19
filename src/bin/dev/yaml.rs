use std::path::PathBuf;

use clap::Subcommand;

use crate::clap::Command;
use dev_cli::{config::Config, yaml};

#[derive(Subcommand)]
pub enum Yaml {
    Update { file: String, target: String },
}

impl Command for Yaml {
    async fn run(&self, _config: &mut Config) -> Result<(), anyhow::Error> {
        match self {
            Yaml::Update { file, target } => {
                let filepath = PathBuf::new().join(file);
                yaml::update(filepath, target).await?;
            }
        }
        Ok(())
    }
}
