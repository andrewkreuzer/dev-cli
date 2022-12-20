use std::fs;
use std::path::Path;

use clap::Subcommand;

use dev_cli::config::Config;
use log::warn;

#[derive(Subcommand)]
pub enum ReposCommand {
    Add {
        file: String,
        destination: String,

        #[clap(short, long)]
        message: String,
    },
    Update,
}

pub async fn handle_repos_command(
    cmd: &Option<ReposCommand>,
    config: &mut Config,
) -> Result<(), anyhow::Error> {
    match cmd {
        Some(ReposCommand::Add {
            file,
            destination,
            message,
        }) => {
            for repo in config.get_repos() {
                let to = match &repo.path {
                    Some(path) => Path::new(&path).join(destination),
                    None => {
                        warn!("{} does not have a path", repo.name);
                        continue;
                    }
                };
                fs::copy(file, to)?;

                repo.add(vec![destination.to_string()], true)?;
                repo.commit(&message)?;
            }
        }
        Some(ReposCommand::Update) => {
            for repo in config.get_repos() {
                if repo.url == None {
                    warn!("{} does not have a url", repo.name);
                    continue;
                }

                repo.checkout("main")?.pull("main")?;
            }
        }
        None => (),
    }

    Ok(())
}
