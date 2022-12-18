use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, error::Error, fmt, io};
use log::{info, warn, trace};

use anyhow::{anyhow, bail};
use git2::{Cred, RemoteCallbacks, Repository, StashFlags};
use serde::{Deserialize, Serialize};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug)]
pub enum GitError {
    Git(git2::Error),
    Io(io::Error),
}

impl Error for GitError {}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GitError::Git(e) => write!(f, "Git error: {}", e),
            GitError::Io(e) => write!(f, "IO error: {}", e),
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

    pub fn open(&self) -> Result<Repository, git2::Error> {
        let path = Path::new(self.path.as_ref().unwrap());
        let repo = Repository::open(path)?;

        Ok(repo)
    }

    pub fn clone_repo(&mut self, path: &str) -> Result<&Self, anyhow::Error> {
        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(callbacks());

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fo);

        let org = match &self.org {
            Some(org) => org.to_string(),
            None => "".to_string()
        };

        info!("Cloning {} in Org: {}", self.name, org);
        let path = PathBuf::new().join(&format!("{}/{}", path, self.name));
        if path.exists() {
            warn!("{} already exists", path.to_str().unwrap());
        } else {
            builder.clone(self.url.as_ref().unwrap(), &path)?;
            self.path = match path.to_str() {
                Some(p) => Some(p.to_string()),
                None => None
            };
        }

        Ok(self)
    }

    pub fn add(&self, files: Vec<String>, update: bool) -> Result<&Self, git2::Error> {
        let repo = self.open()?;
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

        Ok(self)
    }

    pub fn stash(&self) -> Result<&Self, git2::Error> {
        let mut repo = self.open()?;

        repo.stash_save(
            &repo.signature()?,
            "stash",
            Some(StashFlags::DEFAULT),
        )?;

        Ok(self)
    }

    pub fn stash_pop(&self) -> Result<&Self, git2::Error> {
        let mut repo = self.open()?;

        repo.stash_pop(0, None)?;

        Ok(self)
    }

    pub fn branch(&self, branch: &str) -> Result<&Self, git2::Error> {
        let repo = self.open()?;
        repo.branch(branch, &repo.head()?.peel_to_commit()?, false)?;

        Ok(self)
    }

    pub fn checkout(&self, branch: &str) -> Result<&Self, git2::Error> {
        let repo = self.open()?;
        let mut cb = git2::build::CheckoutBuilder::new();
        cb.force();

        let obj = repo.revparse_single(branch)?;
        repo.checkout_tree(&obj, Some(&mut cb))?;
        let refname = format!("refs/heads/{}", branch);
        repo.set_head(&refname)?;

        Ok(self)
    }

    pub fn get_remote(&self) -> Result<String, git2::Error> {
        let repo = self.open()?;
        let remote = repo.find_remote("origin")?;
        let url = remote.url().unwrap();

        Ok(url.to_string())
    }

    pub fn commit(&self, commit_message: &str) -> Result<&Self, git2::Error> {
        let repo = self.open()?;
        let mut index = repo.index()?;
        let oid = index.write_tree()?;
        let tree = repo.find_tree(oid)?;
        let parent_commit = repo.head()?.peel_to_commit()?;

        repo.commit(
            Some("HEAD"),
            &repo.signature()?,
            &repo.signature()?,
            commit_message,
            &tree,
            &[&parent_commit],
        )?;

        Ok(self)
    }

    pub fn push(&self) -> Result<&Self, anyhow::Error> {
        let git_repo = self.open()?;
        let head = git_repo.head()?;

        let refname = match head.name() {
            Some(name) => name,
            None => bail!("head has no name")
        };

        let mut origin = git_repo.find_remote("origin")?;
        let mut po = git2::PushOptions::new();
        po.remote_callbacks(callbacks());
        origin.push(&[format!("{refname}:{refname}")], Some(&mut po))?;

        Ok(self)
    }

    pub fn pull(&self, branch: &str) -> Result<&Self, anyhow::Error> {
        let git_repo = self.open()?;
        let mut remote = git_repo.find_remote("origin")?;
        let fetch_commit = do_fetch(&git_repo, &[branch], &mut remote)?;
        do_merge(&git_repo, &branch, fetch_commit)?;

        Ok(self)
    }
}

fn do_fetch<'a>(
    repo: &'a git2::Repository,
    refs: &[&str],
    remote: &'a mut git2::Remote,
) -> Result<git2::AnnotatedCommit<'a>, git2::Error> {
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks());
    fo.download_tags(git2::AutotagOption::All);
    println!("Fetching {} for repo", remote.name().unwrap());
    remote.fetch(refs, Some(&mut fo), None)?;

    let stats = remote.stats();
    if stats.local_objects() > 0 {
        println!(
            "\rReceived {}/{} objects in {} bytes (used {} local \
                objects)",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes(),
            stats.local_objects()
        );
    } else {
        println!(
            "\rReceived {}/{} objects in {} bytes",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes()
        );
    }

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    Ok(repo.reference_to_annotated_commit(&fetch_head)?)
}

fn fast_forward(
    repo: &Repository,
    lb: &mut git2::Reference,
    rc: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
    println!("{}", msg);
    lb.set_target(rc.id(), &msg)?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default()
            // For some reason the force is required to make the working directory actually get updated
            // I suspect we should be adding some logic to handle dirty working directory states
            // but this is just an example so maybe not.
            .force(),
    ))?;

    Ok(())
}

fn normal_merge(
    repo: &Repository,
    local: &git2::AnnotatedCommit,
    remote: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        println!("Merge conficts detected...");
        repo.checkout_index(Some(&mut idx), None)?;
        return Ok(());
    }
    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
    let msg = format!("Merge: {} into {}", remote.id(), local.id());
    let sig = repo.signature()?;
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;
    let _merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;
    repo.checkout_head(None)?;

    Ok(())
}

fn do_merge<'a>(
    repo: &'a Repository,
    remote_branch: &str,
    fetch_commit: git2::AnnotatedCommit<'a>,
) -> Result<(), git2::Error> {
    let analysis = repo.merge_analysis(&[&fetch_commit])?;

    if analysis.0.is_fast_forward() {
        println!("Doing a fast forward");
        let refname = format!("refs/heads/{}", remote_branch);
        match repo.find_reference(&refname) {
            Ok(mut r) => {
                fast_forward(repo, &mut r, &fetch_commit)?;
            }
            Err(_) => {
                repo.reference(
                    &refname,
                    fetch_commit.id(),
                    true,
                    &format!("Setting {} to {}", remote_branch, fetch_commit.id()),
                )?;
                repo.set_head(&refname)?;
                repo.checkout_head(Some(
                    git2::build::CheckoutBuilder::default()
                        .allow_conflicts(true)
                        .conflict_style_merge(true)
                        .force(),
                ))?;
            }
        };
    } else if analysis.0.is_normal() {
        let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
        normal_merge(&repo, &head_commit, &fetch_commit)?;
    } else {
        println!("Nothing to do...");
    }

    Ok(())
}

fn callbacks() -> RemoteCallbacks<'static> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.transfer_progress(|stats| {
        if stats.received_objects() == stats.total_objects() {
            print!(
                "Resolving deltas {}/{}\r",
                stats.indexed_deltas(),
                stats.total_deltas()
            );
        } else if stats.total_objects() > 0 {
            print!(
                "Received {}/{} objects ({}) in {} bytes\r",
                stats.received_objects(),
                stats.total_objects(),
                stats.indexed_objects(),
                stats.received_bytes()
            );
        }
        io::stdout().flush().unwrap();
        true
    });

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
    depth: usize,
    recurse: bool,
) -> Result<Vec<(PathBuf, Repository)>, anyhow::Error> {
    let mut directories = vec![];

    if recurse {
        directories = scan_directories(directory, depth);
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

fn scan_directories(starting_point: &Path, depth: usize) -> Vec<(PathBuf, Repository)> {
    WalkDir::new(starting_point)
        .max_depth(depth)
        .into_iter()
        .filter_entry(|e| is_not_hidden(e))
        .filter_map(|v| v.ok())
        .map(|x| x.into_path())
        .filter_map(|p| is_directory(p))
        .flat_map(|y| scan_directory(&y))
        .collect::<Vec<(PathBuf, Repository)>>()
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
