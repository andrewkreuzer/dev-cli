use std::env;

use anyhow::bail;
use clap::Subcommand;

use dev_cli::config::Config;
use crate::clap::Command;

#[derive(Subcommand)]
pub enum Git {
    Test,
    Add {
        repo: Option<String>,
        files: Vec<String>,
    },
    Commit {
        repo: Option<String>,

        #[clap(short, long)]
        message: String,
    },
    Push {
        repo: Option<String>,
    },
    Pull {
        repo: Option<String>,
        branch: Option<String>,
    },
    Fetch {
        repo: Option<String>,
        branch: Option<String>,
    },
}

impl Command for Git {
    async fn run(&self, config: &mut Config) -> Result<(), anyhow::Error> {
    let cwd_pathbuf = env::current_dir()?;

    let cwd = cwd_pathbuf
        .file_name()
        .and_then(|d| d.to_str())
        .expect("error getting current working directory");

    match self {
        Git::Test => {
            match config.get_repo("graphql-tests") {
                Some(repo) => {
                    println!("{}", repo.default_branch()?);
                }
                None => bail!("repo not found")
            }
        }
        Git::Add { repo, files } => {
            let repo = match repo {
                Some(repo) => repo,
                None => cwd,
            };

            match config.get_repo(repo) {
                Some(git_repo) => git_repo.add(files.to_vec(), false)?,
                None => bail!("Repo not in config"),
            };
        }
        Git::Commit { repo, message } => {
            let repo = match repo {
                Some(repo) => repo,
                None => cwd,
            };

            match config.get_repo(repo) {
                Some(git_repo) => git_repo.commit(message)?,
                None => bail!("Repo not in config"),
            };
        }
        Git::Push { repo } => {
            let repo = match repo {
                Some(repo) => repo,
                None => cwd,
            };

            match config.get_repo(repo) {
                Some(git_repo) => git_repo.push()?,
                None => bail!("Repo not in config"),
            };
       }
        Git::Pull { repo, branch } => {
            let repo = match repo {
                Some(repo) => repo,
                None => cwd,
            };

            let branch = match branch {
                Some(branch) => branch,
                None => "main",
            };

            match config.get_repo(repo) {
                Some(git_repo) => git_repo.pull(Some(branch))?,
                None => bail!("Repo not in config"),
            };
        }
        Git::Fetch { repo, branch } => {
            let repo = match repo {
                Some(repo) => repo,
                None => cwd,
            };

            let branch = branch.as_ref().map(|branch| branch.as_str());

            match config.get_repo(repo) {
                Some(git_repo) => git_repo.fetch(branch)?,
                None => bail!("Repo not in config"),
            };
        }
    }

    Ok(())
    }
}
