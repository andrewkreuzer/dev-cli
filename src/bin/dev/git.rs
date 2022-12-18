use std::env;

use anyhow::bail;
use clap::Subcommand;

use dev_cli::config::Config;

#[derive(Subcommand)]
pub enum GitCommand {
    Add {
        repo: Option<String>,
        files: Vec<String>,
    },
    Commit {
        repo: Option<String>,

        #[clap(short, long)]
        message: String
    },
    Push {
        repo: Option<String>,
    },
    Pull {
        repo: Option<String>,
        branch: Option<String>,
    },
}

pub async fn handle_git_command(
    cmd: &Option<GitCommand>,
    config: &mut Config,
) -> Result<(), anyhow::Error> {
    match cmd {
        Some(GitCommand::Add { repo, files }) => {
            let directory = env::current_dir()
                .expect("error getting current directory");

            let repo = match repo {
                Some(repo) => repo.clone(),
                None => directory.display().to_string(),
            };

            match config.get_repo(&repo) {
                Some(git_repo) => git_repo.add(files.to_vec(), false)?,
                None => bail!("Repo not in config")
            };
        }
        Some(GitCommand::Commit { repo, message }) => {
            let directory = env::current_dir()
                .expect("error getting current directory");

            let repo = match repo {
                Some(repo) => repo.to_string(),
                None => directory.display().to_string(),
            };

            config.get_repo(&repo).unwrap().commit(&message)?;
        }
        Some(GitCommand::Push { repo }) => {
            // TODO: this will fail miserably because it's the absolue path
            let directory = env::current_dir()
                .expect("error getting current directory");

            let repo = match repo {
                Some(repo) => repo.to_string(),
                None => directory.display().to_string(),
            };

            config.get_repo(&repo).unwrap().push()?;
        }
        Some(GitCommand::Pull { repo, branch }) => {
            let directory = env::current_dir()
                .expect("error getting current directory");

            let repo = match repo {
                Some(repo) => repo.to_string(),
                None => directory.display().to_string(),
            };

            let branch = match branch {
                Some(branch) => branch,
                None => "main",
            };

            config.get_repo(&repo).unwrap().pull(branch)?;
        }
        None => (),
    }

    Ok(())
}
