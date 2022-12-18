use std::{path::PathBuf, fs::{File, self}};
use env_logger::Target;
use log::{warn, info, LevelFilter};

use clap::{Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, InfoLevel};

use crate::scan::handle_scan_command;
use super::repo::handle_repo_command;
use super::repos::handle_repos_command;
use super::git::handle_git_command;
use super::github::handle_github_command;
use dev_cli::{config, yaml};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(flatten)]
    verbose: Verbosity<InfoLevel>,

    #[clap(short, long, value_parser, value_name = "FILE")]
    config: Option<PathBuf>,

    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum YamlCommand {
    Update { file: String, target: String },
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[clap(short, long)]
        global: bool,
    },
    Git {
        #[clap(subcommand)]
        cmd: Option<super::git::GitCommand>,
    },
    Github,
    Scan {
        directory: Option<PathBuf>,

        #[clap(short, long, default_value = "1")]
        depth: usize,

        #[clap(short, long)]
        recurse: bool,

        #[clap(short, long)]
        add: bool,
    },
    Yaml {
        #[clap(subcommand)]
        cmd: Option<YamlCommand>,
    },
    Repo {
        #[clap(subcommand)]
        cmd: Option<super::repo::RepoCommand>
    },
    Repos {
        #[clap(subcommand)]
        cmd: Option<super::repos::ReposCommand>
    }
}

pub async fn init() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    log(cli.verbose.log_level_filter())?;

    let config_path: PathBuf = match cli.config.as_deref() {
        Some(path) => path.to_path_buf(),
        None => PathBuf::new().join("dev.toml"),
    };

    let mut config = config::load(config_path)?;

    match &cli.command {
        Some(Commands::Init { global }) => {
            let path = PathBuf::new().join("dev.toml");
            if !path.exists() {
                info!("Creating new config");
                config::create_new(&path)?;
            } else {
                warn!("Local config already exists");
            }

            if *global {
                config.save_global()?
            }
        }
        Some(Commands::Repo { cmd }) => {
            handle_repo_command(cmd, &mut config).await?;
        }
        Some(Commands::Repos { cmd }) => {
            handle_repos_command(cmd, &mut config).await?;
        }
        Some(Commands::Github) => {
            handle_github_command(config).await?;
        }
        Some(Commands::Git { cmd }) => {
            handle_git_command(cmd, &mut config).await?;
        }
        Some(Commands::Scan { directory, depth, recurse, add }) => {
            handle_scan_command(
                directory.clone(),
                *depth,
                *recurse,
                *add,
                &mut config
            ).await?;
        }
        Some(Commands::Yaml { cmd }) => {
            if let Some(YamlCommand::Update { file, target }) = cmd {
                let filepath = PathBuf::new().join(file);
                yaml::update(filepath, target).await?
            }
        }
        None => (),
    };

    Ok(())
}

fn log(log_level: LevelFilter) -> Result<(), anyhow::Error> {
    let cache_dir = dirs::cache_dir().unwrap().join("dev");
    if !cache_dir.is_dir() {
        fs::create_dir(&cache_dir)?;
    }

    let log_file = cache_dir.join("dev.log");
    let file: File;
    if !log_file.is_file() {
        file = File::create(log_file)?;
    } else {
        file = File::options()
            .append(true)
            .create(true)
            .open(log_file)?;
    }

    env_logger::Builder::new()
        .filter_level(log_level)
        .target(Target::Pipe(Box::new(file)))
        .init();

    Ok(())
}
