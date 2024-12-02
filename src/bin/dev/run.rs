use crate::clap::Command;
use anyhow::anyhow;
use clap::Args;
use dev_cli::config::Config;
use dev_cli::lang::{Dev, Language, LanguageFunctions};
use dev_cli::utils::write_tmp_file;
use log::debug;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct Run {
    #[arg(short, long, help = "file to run based on extension")]
    pub file: Option<String>,
    #[arg(
        short,
        long,
        requires = "file",
        help = "file type to run, if ommited will use file extension"
    )]
    pub type_: Option<String>,
    #[arg(conflicts_with_all = ["file", "type_"], help = "command in config file to run")]
    pub name: Option<String>,
    #[arg(short, long, help = "arguments to pass to command")]
    pub args: Vec<String>,
}

impl Command for Run {
    async fn run(&self, config: &mut Config) -> Result<(), anyhow::Error> {
        let dev = Dev::new(config);
        let args = self.args.iter().map(|s| s as &str).collect::<Vec<&str>>();

        match (&self.type_, &self.file) {
            (Some(_), None) => {
                return Err(anyhow!("No file provided"));
            }
            (None, Some(file)) => {
                let runner = Language::try_from(file.as_str())?;
                let status = runner.run_file(dev, file, args).await?;
                debug!("{status}");
                return Ok(());
            }
            (Some(t), Some(file)) => {
                let runner = Language::try_from(t.as_str())?;
                let status = runner.run_file(dev.clone(), file, args).await?;
                debug!("{status}");
                return Ok(());
            }
            (None, None) => {}
        }

        let name = match &self.name {
            Some(name) => name,
            None => {
                return Err(anyhow!("No name provided"));
            }
        };

        run_alias(config, name, Some(args)).await
    }
}

pub async fn run_alias(
    config: &Config,
    alias: &str,
    args: Option<Vec<&str>>,
) -> Result<(), anyhow::Error> {
    let args = args.unwrap_or_default();

    let runref = config
        .get_run(alias)
        .ok_or(anyhow!("{alias} command not found in {}", alias))?;

    let lang = runref
        .filetype
        .as_ref()
        .ok_or(anyhow!("runner ref filetype not found"))?;

    let dev = Dev::new(config);
    let file = runref.file.as_ref();
    let command = runref.command.as_ref();
    if let Some(f) = file {
        let dev = Dev::new(config);
        let status = lang.run_file(dev, f, vec![]).await?;
        debug!("status: {}", status);
    }

    if let Some(c) = command {
        let tmpfilepath = format!("{}{}", config.get_tmp_dir(), lang.get_extension());
        write_tmp_file(tmpfilepath.as_str(), c, true)?;
        let status = lang.run_file(dev, tmpfilepath.as_str(), args).await?;
        debug!("status: {}", status);
    }
    Ok(())
}
