use std::path::{Path, PathBuf};
use std::{env, error::Error, fmt, io};
use log::{info, warn, trace};

use anyhow::{anyhow, bail};
use git2::{Cred, Oid, RemoteCallbacks, Repository};
use serde::{Deserialize, Serialize};
use walkdir::{DirEntry, WalkDir};

use crate::config::Root;

#[derive(Debug)]
pub enum GitError {
    Git(git2::Error),
    Io(io::Error),
}

impl Error for GitError {}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitError::Io(e) => e.fmt(f),
            GitError::Git(e) => e.fmt(f),
        }
    }
}

impl From<git2::Error> for GitError {
    fn from(e: git2::Error) -> Self {
        GitError::Git(e)
    }
}

impl From<io::Error> for GitError {
    fn from(e: io::Error) -> Self {
        GitError::Io(e)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GitRepository {
    pub org: Option<String>,
    pub name: String,
    pub url: Option<String>,
    pub path: Option<String>,
}

impl PartialEq<Root> for GitRepository {
    fn eq(&self, other: &Root) -> bool {
        self.name == other.name
        || self.url == other.url
    }
}

impl GitRepository {
    pub fn new(full_name: &str, path: Option<&str>) -> Result<Self, anyhow::Error> {
        let names: Vec<&str> = full_name.split("/").collect();

        if names.len() != 2 {
            bail!("repo name must consist of {{org}}/{{repo}}");
        }

        let org = Some(names.get(0).unwrap().to_string());
        let name = names.get(1).unwrap().to_string();
        let path = match path {
            Some(path) => Some(path.to_string()),
            None => None,
        };

        Ok(GitRepository {
            org,
            name,
            path,
            url: Some(format!("git@github.com:{}", full_name)),
        })
    }

    pub fn clone_repo(&mut self) -> Result<(), anyhow::Error> {
        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(ssh_callbacks());

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fo);

        let org = match &self.org {
            Some(org) => org.to_string(),
            None => "".to_string()
        };
        info!("Cloning {} in Org: {}", self.name, org);
        let path = PathBuf::new().join(&format!("{}", self.name));
        if path.exists() {
            warn!("{} already exists", path.to_str().unwrap());
        } else {
            builder.clone(self.url.as_ref().unwrap(), &path)?;
            self.path = match path.to_str() {
                Some(p) => Some(p.to_string()),
                None => None
            };
        }

        Ok(())
    }

    pub fn add(&self, files: Vec<String>, update: bool) -> Result<(), git2::Error> {
        let path = Path::new(self.path.as_ref().unwrap());
        let repo = Repository::open(path)?;
        let mut index = repo.index()?;

        let cb = &mut |path: &Path, _matched_spec: &[u8]| -> i32 {
            let status = repo.status_file(path).unwrap();

            if status.contains(
                git2::Status::WT_MODIFIED)
                || status.contains(git2::Status::WT_NEW
            ) {
                info!("add '{}'", path.display());
                0
            } else {
                1
            }
        };

        let cb = if update {
            Some(cb as &mut git2::IndexMatchedPath)
        } else {
            None
        };

        index.add_all(files.iter(), git2::IndexAddOption::DEFAULT, cb)?;
        index.write()?;

        Ok(())
    }

    pub fn commit(&self) -> Result<Oid, git2::Error> {
        let path = Path::new(self.path.as_ref().unwrap());
        let repo = Repository::open(path)?;
        let mut index = repo.index()?;

        let oid = index.write_tree()?;
        let tree = repo.find_tree(oid)?;
        let parent_commit = repo.head()?.peel_to_commit()?;
        repo.commit(
            Some("HEAD"),
            &repo.signature()?,
            &repo.signature()?,
            "test commit",
            &tree,
            &[&parent_commit],
        )
    }
}

fn ssh_callbacks() -> RemoteCallbacks<'static> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            Path::new(
                &format!("{}/.ssh/id_ed25519",
                    env::var("HOME").unwrap())
            ),
            None,
        )
    });

    callbacks
}

pub fn scan(
    directory: &Path,
    recurse: bool,
) -> Result<Vec<(PathBuf, Repository)>, anyhow::Error> {
    let mut directories = vec![];
    if recurse {
        directories = scan_directories(directory);
    } else {
        let repo = scan_directory(directory)?;
        directories.push(repo);
    }

    if directories.len() > 0 {
        Ok(directories)
    } else {
        Err(anyhow!("No directories found"))
    }
}

fn scan_directory(
    directory: &Path
) -> Result<(PathBuf, Repository), anyhow::Error> {
    match Repository::open(directory) {
        Ok(repo) => {
            info!("Found repo at {:?}", directory.file_name().unwrap());
            Ok((directory.into(), repo))
        }
        Err(e) => {
            trace!("No repo found at {:?}", directory.file_name().unwrap());
            Err(anyhow!(GitError::Git(e)))
        }
    }
}

fn is_not_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() == 0 || !s.starts_with("."))
        .unwrap_or(false)
}

fn is_directory(path: PathBuf) -> Option<PathBuf> {
    if path.is_dir() {
        Some(path)
    } else {
        None
    }
}

fn scan_directories(starting_point: &Path) -> Vec<(PathBuf, Repository)> {
    WalkDir::new(starting_point)
        .into_iter()
        .filter_entry(|e| is_not_hidden(e))
        .filter_map(|v| v.ok())
        .map(|x| x.into_path())
        .filter_map(|p| is_directory(p))
        .flat_map(|y| scan_directory(&y))
        .collect::<Vec<(PathBuf, Repository)>>()
}
