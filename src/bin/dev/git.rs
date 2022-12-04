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
    Commit,
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
        Some(GitCommand::Commit) => {
            config.get_repo("repo_1").unwrap().commit()?;
        }
        None => (),
    }

    Ok(())
}
