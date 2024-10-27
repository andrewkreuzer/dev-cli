use log::{debug, warn};
use std::{
    collections::{hash_map::Values, HashMap},
    env, error, fmt, fs,
    fs::File,
    io,
    io::prelude::*,
    path::PathBuf,
};

use dirs;
use serde::{Deserialize, Serialize};

use crate::{git::GitRepository, lang::Language};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    TomlDe(toml::de::Error),
    TomlSer(toml::ser::Error),
    Duplicate(String),
    NotFound,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => e.fmt(f),
            Error::TomlDe(e) => e.fmt(f),
            Error::TomlSer(e) => e.fmt(f),
            Error::Duplicate(e) => e.fmt(f),
            Error::NotFound => self.fmt(f),
        }
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::TomlDe(e)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(e: toml::ser::Error) -> Self {
        Error::TomlSer(e)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    filepath: Option<PathBuf>,
    repos: HashMap<String, GitRepository>,
    run: HashMap<String, RunRef>,
    #[serde(alias = "env")]
    environment: Option<Vec<String>>,

    #[serde(skip)]
    tmp_dir: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RunRef {
    pub file: Option<String>,
    pub command: Option<String>,
    pub filetype: Option<Language>,
    pub path: Option<PathBuf>,
    #[serde(alias = "deps")]
    pub dependencies: Option<Vec<String>>,
    #[serde(alias = "env")]
    pub environment: Option<Vec<String>>,
}

impl Config {
    pub fn new(repositories: Option<Vec<GitRepository>>) -> Self {
        let mut repos = HashMap::new();
        let run = HashMap::new();

        if let Some(repositories) = repositories {
            for r in repositories.into_iter() {
                repos.insert(r.name.clone(), r);
            }
        }

        Config {
            repos,
            run,
            filepath: None,
            environment: None,

            tmp_dir: "/tmp/dev".to_string(),
        }
    }

    pub fn set_filepath(&mut self, filepath: PathBuf) {
        self.filepath = Some(filepath);
    }

    pub fn get_filepath(&self) -> &PathBuf {
        self.filepath.as_ref().unwrap()
    }

    pub fn get_repo(&self, repo: &str) -> Option<&GitRepository> {
        self.repos.get(repo)
    }

    pub fn get_repo_map(&self) -> &HashMap<String, GitRepository> {
        &self.repos
    }

    pub fn get_repos(&self) -> Values<String, GitRepository> {
        self.repos.values()
    }

    pub fn get_mut_repo(&mut self, repo: &str) -> Option<&mut GitRepository> {
        self.repos.get_mut(repo)
    }

    pub fn get_env_vars(&self) -> Option<&Vec<String>> {
        self.environment.as_ref()
    }

    pub fn update_repo(&mut self, repo: GitRepository) -> Result<(), Error> {
        self.repos.insert(repo.name.clone(), repo);
        let directory = env::current_dir().expect("error getting current directory");

        write_file(&PathBuf::new().join(directory).join("dev.toml"), self)?;

        Ok(())
    }
    pub fn update(&self) -> Result<(), Error> {
        let directory = env::current_dir().expect("error getting current directory");

        write_file(&PathBuf::new().join(directory).join("dev.toml"), self)?;

        Ok(())
    }

    pub fn add_repo(
        &mut self,
        name: Option<String>,
        git_repo: &GitRepository,
    ) -> Result<&Self, anyhow::Error> {
        for (name, repo) in self.repos.iter() {
            if name == &git_repo.name {
                warn!("{} is duplicate", repo.name);
            }
        }

        let name = match name {
            Some(name) => name,
            None => git_repo.name.clone(),
        };

        self.repos.insert(name, git_repo.to_owned());

        let directory = env::current_dir().expect("error getting current directory");

        write_file(&PathBuf::new().join(directory).join("dev.toml"), self)?;

        Ok(self)
    }

    pub fn save_global(&self) -> Result<(), Error> {
        let config_dir = dirs::config_dir().unwrap().join("dev");
        if !config_dir.is_dir() {
            fs::create_dir(&config_dir)?;
        }

        let config_file = config_dir.join("dev.toml");
        if !config_file.is_file() {
            create_new(&config_file)?;
        } else {
            write_file(&config_file, self)?;
        }

        Ok(())
    }

    pub fn get_run(&self, name: &str) -> Option<&RunRef> {
        self.run.get(name)
    }

    pub fn get_tmp_dir(&self) -> &str {
        &self.tmp_dir
    }
}

pub fn create_new(filepath: &PathBuf) -> Result<Config, Error> {
    let _file = File::create(filepath)?;
    write_file(filepath, &Config::new(None))
}

pub fn load(filepath: PathBuf) -> Result<Config, Error> {
    match read_file(&filepath) {
        Ok(content) => match toml::from_str::<Config>(&content) {
            Ok(mut config) => {
                config.set_filepath(filepath);
                Ok(config)
            }
            Err(e) => Err(Error::TomlDe(e)),
        },
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => {
                debug!("No config found in this directory, using default settings");
                Ok(Config::new(None))
            }
            _ => Err(Error::Io(err)),
        },
    }
}

fn read_file(filepath: &PathBuf) -> Result<String, io::Error> {
    if filepath.is_file() {
        let mut file = fs::File::open(filepath)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        Ok(content)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} not found", filepath.to_str().unwrap()),
        ))
    }
}

// TODO: should we dissolve the idea of writing back to the config
//       I feel like it's only going to cause pain and suffering.
//       maybe just for scanning and adding but we'll have to be
//       explicit about the fact that comments, formatting, and a
//       bunch of random shit may happen
fn write_file(filepath: &PathBuf, config: &Config) -> Result<Config, Error> {
    let file = File::options().write(true).truncate(true).open(filepath);

    let toml_str = toml::to_string(&config)?;
    file?.write_all(toml_str.as_bytes())?;

    Ok(config.to_owned())
}
