use std::fs::File;
use std::io::Write;

use crate::clap::Command;
use anyhow::anyhow;
use clap::Args;
use dev_cli::config::Config;
use dev_cli::lang::{Dev, Language, LanguageFunctions};
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
        let dev = Dev::new();
        let (type_, file, name) = (&self.type_, &self.file, &self.name);
        let args = self.args.iter().map(|s| s as &str).collect::<Vec<&str>>();

        match (type_, file) {
            (Some(_), None) => {
                return Err(anyhow!("No file provided"));
            }
            (None, Some(file)) => {
                let runner = Language::try_from(file.as_str())?;
                let status = runner.run_file(dev, file, args).await?;
                debug!("status: {}", status);
                return Ok(());
            }
            (Some(t), Some(file)) => {
                let runner = Language::try_from(t.as_str())?;
                let status = runner.run_file(dev.clone(), file, args).await?;
                debug!("status: {}", status);
                return Ok(());
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
        let lang = runref.filetype.as_ref().ok_or(anyhow!("filetype issue"))?;

        if let Some(file) = runref.file.as_ref() {
            let status = lang.run_file(dev, file, args).await?;
            debug!("status: {}", status);
            return Ok(());
        }

        if let Some(command) = runref.command.as_ref() {
            let ext = lang.get_extension();
            let tmpfilepath = "/tmp/dev".to_string() + ext;
            let file = File::create(tmpfilepath.clone());
            file?.write_all(command.as_bytes())?;

            let file = File::open(tmpfilepath.clone());
            let mut permissions = file?.metadata()?.permissions();
            use std::os::unix::fs::PermissionsExt;
            permissions.set_mode(0o755);
            std::fs::set_permissions(tmpfilepath.clone(), permissions)?;
            lang
                .run_file(dev, tmpfilepath.as_str(), args)
                .await?;
        }

        Ok(())
    }
}
