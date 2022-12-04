use clap::Subcommand;

use dev_cli::{
    config::Config,
    git,
};

#[derive(Subcommand)]
pub enum RepoCommand {
    Clone {
        repo: Option<String>,
    },
    Add {
        name: String,
    },
}

pub async fn handle_repo_command(
    cmd: &Option<RepoCommand>,
    config: &mut Config
) -> Result<(), anyhow::Error> {
    match cmd {
        Some(RepoCommand::Clone { repo }) => {
            if let Some(repo) = repo {
                let mut git_repo = match config.get_repo(repo) {
                    Some(r) => r.to_owned(),
                    None => git::GitRepository::new(repo, None)?,
                };

                git_repo.clone_repo()?;
                config.update_repo(git_repo)?;
            }
        }
        Some(RepoCommand::Add { name }) => {
            let git_repo = git::GitRepository::new(name, None)?;
            config.add_repo(Some(name.to_string()), &git_repo)?;
        }
        None => (),
    }

    Ok(())
}
