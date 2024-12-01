use std::{
    borrow::BorrowMut,
    fs::{self, File},
    path::PathBuf,
    io::Write,
};

use env_logger::Target;
use log::LevelFilter;

use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};

use crate::{
    git::Git,
    github::Github,
    init::Init,
    repo::{Repo, Repos},
    run::Run,
    scan::Scan,
    shell::Shell,
    yaml::Yaml,
};
use dev_cli::config;
use dev_cli::lang::{Dev, LanguageFunctions};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[clap(flatten)]
    verbose: Verbosity<InfoLevel>,

    #[clap(short, long, value_parser, value_name = "FILE")]
    config: Option<PathBuf>,

    #[clap(subcommand)]
    command: Option<Commands>,

    arg: Option<String>,
}

pub trait Command {
    async fn run(&self, config: &mut config::Config) -> Result<(), anyhow::Error>;
}

#[derive(Subcommand)]
enum Commands {
    Init(Init),
    #[clap(subcommand)]
    Git(Git),
    Github(Github),
    Scan(Scan),
    #[clap(subcommand)]
    Yaml(Yaml),
    #[clap(subcommand)]
    Repo(Repo),
    #[clap(subcommand)]
    Repos(Repos),
    Run(Run),
    Shell(Shell),
}

pub async fn init() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    log(cli.verbose.log_level_filter())?;

    let config_path: PathBuf = match cli.config {
        Some(path) => path.to_path_buf(),
        None => PathBuf::new().join("dev.toml"),
    };

    let mut config = config::load(config_path)?;
    let cfg = config.borrow_mut();
    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Init(cmd) => cmd.run(cfg).await?,
            Commands::Git(cmd) => cmd.run(cfg).await?,
            Commands::Github(cmd) => cmd.run(cfg).await?,
            Commands::Scan(cmd) => cmd.run(cfg).await?,
            Commands::Yaml(cmd) => cmd.run(cfg).await?,
            Commands::Repo(cmd) => cmd.run(cfg).await?,
            Commands::Repos(cmd) => cmd.run(cfg).await?,
            Commands::Run(cmd) => cmd.run(cfg).await?,
            Commands::Shell(cmd) => cmd.run(cfg).await?,
        }
    } else if let Some(arg) = cli.arg {
        if let Some(runref) = config.get_run(&arg) {
            if let Some(runner) = &runref.filetype {
                let dev = Dev::new(&config);
                if let Some(f) = &runref.file {
                    runner.run_file(dev, f, vec![]).await?;
                } else if let Some(command) = runref.command.as_ref() {
                    let tmpfilepath = format!("{}{}", config.get_tmp_dir(), runner.get_extension());
                    {
                        // make sure file is out of scope so we don't get
                        // "text file busy" error
                        let mut file = File::create(tmpfilepath.clone())?;
                        file.write_all(command.as_bytes())?;

                        let mut permissions = file.metadata()?.permissions();
                        use std::os::unix::fs::PermissionsExt;
                        permissions.set_mode(0o755);
                        std::fs::set_permissions(tmpfilepath.clone(), permissions)?;
                    }
                    let args = vec![];
                    let _status = runner.run_file(dev, tmpfilepath.as_str(), args).await?;
                }
            }
        } else {
            println!("nana");
        }
    };

    Ok(())
}

fn log(log_level: LevelFilter) -> Result<(), anyhow::Error> {
    let cache_dir = dirs::cache_dir().unwrap().join("dev");
    if !cache_dir.is_dir() {
        fs::create_dir(&cache_dir)?;
    }

    let log_file = cache_dir.join("dev.log");
    let file = File::options().append(true).create(true).open(log_file)?;

    env_logger::Builder::new()
        .filter_level(log_level)
        .target(Target::Pipe(Box::new(file)))
        .target(Target::Stdout)
        .init();

    Ok(())
}
