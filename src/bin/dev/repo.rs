use std::fs;
use std::path::Path;

use clap::Subcommand;

use log::warn;

use crate::clap::Command;
use dev_cli::{config::Config, git};

#[derive(Subcommand)]
#[command(arg_required_else_help = true)]
pub enum Repo {
    Clone {
        repo: Option<String>,

        #[clap(short, long, default_value = ".")]
        path: String,
    },
    Add {
        name: String,
    },
}

impl Repo {
    pub async fn run(&self, config: &mut Config) -> Result<(), anyhow::Error> {
        match self {
            Repo::Clone {
                repo: Some(repo),
                path,
            } => {
                let mut git_repo = match config.get_repo(repo) {
                    Some(r) => r.to_owned(),
                    None => git::GitRepository::new(repo, None)?,
                };

                git_repo.clone_repo(path)?;
                config.update_repo(git_repo)?;
            }
            Repo::Add { name } => {
                let git_repo = git::GitRepository::new(name, None)?;
                config.add_repo(Some(name.to_string()), &git_repo)?;
            }
            _ => (),
        }

        Ok(())
    }
}

#[derive(Subcommand)]
pub enum Repos {
    Add {
        file: String,
        destination: String,

        #[clap(short, long)]
        message: String,
    },
    Update,
}

impl Command for Repos {
    async fn run(&self, config: &mut Config) -> Result<(), anyhow::Error> {
        match self {
            Repos::Add {
                file,
                destination,
                message,
            } => {
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
                    repo.commit(message)?;
                }
            }
            Repos::Update => {
                for repo in config.get_repos() {
                    println!("Running update on {}", repo.name);

                    if repo.url.is_none() {
                        warn!("{} does not have a url", repo.name);
                        continue;
                    }

                    let default_branch = repo.default_branch()?;
                    repo.checkout(&default_branch)?
                        .pull(Some(&default_branch))?;

                    println!();
                }
            }
        }

        Ok(())
    }
}
