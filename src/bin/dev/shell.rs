use clap::Args;

use crate::clap::Command;
use dev_cli::{
    config::Config,
    runners::{Language, LanguageFunctions},
};

#[derive(Args)]
pub struct Shell {
    pub name: Option<String>,
}

impl Command for Shell {
    async fn run(&self, _config: &mut Config) -> Result<(), anyhow::Error> {
        if let Some(name) = &self.name {
            let runner = Language::try_from(name.as_str())?;
            runner.run_shell("ls", [].into()).await?;
        }
        Ok(())
    }
}
