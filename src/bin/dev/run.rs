use anyhow::anyhow;
use clap::Args;
use dev_cli::config::Config;
use dev_cli::lang::{Dev, Language, LanguageFunctions};
use crate::clap::Command;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct Run {
    #[arg(short, long, help = "file to run based on extension")]
    pub file: Option<String>,
    #[arg(short, long, help = "file type to run: ['py', 'lua', 'sh', 'zsh']")]
    pub type_: Option<String>,
    #[arg(help = "command in config file to run")]
    pub name: Option<String>,
}

impl Command for Run {
    async fn run(&self, config: &mut Config) -> Result<(), anyhow::Error> {
        let dev = Dev::new();
        let (type_, file, name) = (&self.type_, &self.file, &self.name);

        match (type_, file) {
            (Some(_), None) => {
                return Err(anyhow!("No file provided"));
            }
            (None, Some(file)) => {
                let runner = Language::try_from(file.as_str())?;
                return runner.run_file(dev, file).await;
            }
            (Some(t), Some(file)) => {
                let runner = Language::try_from(t.as_str())?;
                return runner.run_file(dev.clone(), file).await;
            }
            (None, None) => {}
        }

        let name = match name {
            Some(name) => name,
            None => {
                return Err(anyhow!("No name provided"));
            }
        };

        let runref = config.get_run(name).ok_or(anyhow!(
            "{name} command not found in {}",
            config.get_filepath().display(),
        ))?;
        let runner = runref.filetype.as_ref().ok_or(anyhow!("filetype issue"))?;

        if let Some(file) = runref.file.as_ref() {
            runner.run_file(dev, file).await?;
        }

        if let Some(command) = runref.command.as_ref() {
            runner.run_shell(command).await?;
        }

        Ok(())
    }
}
