use log::{error, info, warn};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, error::Error, fmt, io};

use anyhow::{anyhow, bail};
use git2::{Cred, RemoteCallbacks, Repository, StashFlags};
use serde::{Deserialize, Serialize};

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

        let org = Some(
            names
                .first()
                .ok_or(anyhow!("failed to get org name"))?
                .to_string(),
        );
        let name = names.get(1).unwrap().to_string();
        let path = path.map(|path| path.to_string());

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

    pub fn current_branch(&self) -> Result<String, anyhow::Error> {
        let git_repo = self.open()?;
        let head = git_repo.head()?;

        head.shorthand()
            .map(|s| s.to_string())
            .ok_or(anyhow!("head has no name"))
    }

    pub fn default_branch(&self) -> Result<String, git2::Error> {
        let git_repo = self.open()?;
        let mut remote = git_repo.find_remote("origin")?;
        remote.connect_auth(git2::Direction::Fetch, Some(callbacks()), None)?;
        let default_branch = remote.default_branch()?;
        remote.disconnect()?;

        let default_branch = match default_branch.as_str() {
            Some(branch) => branch,
            None => return Err(git2::Error::from_str("no default branch")),
        };

        Ok(default_branch.split("/").last().unwrap().to_string())
    }

    pub fn remote(&self) -> Result<String, git2::Error> {
        let repo = self.open()?;
        let remote = repo.find_remote("origin")?;
        let url = remote.url().unwrap();

        Ok(url.to_string())
    }

    pub fn clone_repo(&mut self, path: &str) -> Result<&Self, anyhow::Error> {
        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(callbacks());

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fo);

        let org = match &self.org {
            Some(org) => org,
            None => "",
        };

        info!("Cloning {} in Org: {}", self.name, org);
        let path = PathBuf::new().join(format!("{}/{}", path, self.name));
        if path.exists() {
            warn!("{} already exists", path.to_str().unwrap());
        } else {
            builder.clone(self.url.as_ref().unwrap(), &path)?;
            self.path = path.to_str().map(|p| p.to_string());
        }

        Ok(self)
    }

    pub fn branch(&self, branch: &str) -> Result<&Self, git2::Error> {
        let repo = self.open()?;
        repo.branch(branch, &repo.head()?.peel_to_commit()?, false)?;

        Ok(self)
    }

    pub fn checkout_default(&self) -> Result<&Self, git2::Error> {
        let repo = self.open()?;
        let mut cb = git2::build::CheckoutBuilder::new();
        cb.force();

        repo.checkout_head(Some(&mut cb))?;

        Ok(self)
    }

    pub fn checkout(&self, branch: &str) -> Result<&Self, anyhow::Error> {
        let repo = self.open()?;
        let mut cb = git2::build::CheckoutBuilder::new();
        cb.force();

        let obj = match repo.revparse_single(branch) {
            Ok(obj) => obj,
            Err(e) => {
                println!("branch {} not found", branch);
                return Err(anyhow!("Unable to revparse: {}, {}", branch, e));
            }
        };

        repo.checkout_tree(&obj, Some(&mut cb))?;
        let refname = format!("refs/heads/{}", branch);

        if let Err(e) = repo.set_head(&refname) {
            println!("Failed to set head to: {}", branch);
            return Err(anyhow!("Failed to set head to {}: {}", branch, e));
        }

        Ok(self)
    }

    pub fn add(&self, files: Vec<String>, update: bool) -> Result<&Self, git2::Error> {
        let repo = self.open()?;
        let mut index = repo.index()?;

        let cb = &mut |path: &Path, _matched_spec: &[u8]| -> i32 {
            let status = repo.status_file(path).unwrap();

            if status.contains(git2::Status::WT_MODIFIED) || status.contains(git2::Status::WT_NEW) {
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
            None => bail!("head has no name"),
        };

        let mut origin = git_repo.find_remote("origin")?;
        let mut po = git2::PushOptions::new();
        po.remote_callbacks(callbacks());
        origin.push(&[format!("{refname}:{refname}")], Some(&mut po))?;

        Ok(self)
    }

    pub fn pull(&self, branch: Option<&str>) -> Result<&Self, anyhow::Error> {
        let git_repo = self.open()?;
        let mut remote = git_repo.find_remote("origin")?;

        let current_branch = self.current_branch()?;
        let branch = match branch {
            Some(branch) => branch,
            None => current_branch.as_str(),
        };

        let fetch_commit = fetch(&git_repo, &[branch], &mut remote)?;

        if let Err(e) = merge(&git_repo, branch, fetch_commit) {
            error!("Failed to merge {}: {}", self.name, e);
        }

        Ok(self)
    }

    pub fn fetch(&self, branch: Option<&str>) -> Result<&Self, anyhow::Error> {
        let git_repo = self.open()?;
        let mut remote = git_repo.find_remote("origin")?;
        match branch {
            Some(branch) => {
                fetch(&git_repo, &[branch], &mut remote)?;
            }
            None => {
                let remote_refspecs = remote.fetch_refspecs()?;
                let refspecs: Vec<&str> = remote_refspecs.iter().flatten().collect();
                fetch(&git_repo, &refspecs, &mut remote)?;
            }
        }

        Ok(self)
    }

    pub fn stash(&self) -> Result<&Self, git2::Error> {
        let mut repo = self.open()?;

        repo.stash_save(&repo.signature()?, "stash", Some(StashFlags::DEFAULT))?;

        Ok(self)
    }

    pub fn stash_pop(&self) -> Result<&Self, git2::Error> {
        let mut repo = self.open()?;

        repo.stash_pop(0, None)?;

        Ok(self)
    }

    pub fn rev_parse(&self, rev: &str) -> Result<String, anyhow::Error> {
        let repo = self.open()?;
        let revspec = repo.revparse(rev)?;

        if revspec.mode().contains(git2::RevparseMode::SINGLE) {
            println!("{}", revspec.from().unwrap().id());
        } else if revspec.mode().contains(git2::RevparseMode::RANGE) {
            let to = revspec.to().unwrap();
            let from = revspec.from().unwrap();
            println!("{}", to.id());

            if revspec.mode().contains(git2::RevparseMode::MERGE_BASE) {
                let base = repo.merge_base(from.id(), to.id())?;
                println!("{}", base);
            }

            println!("^{}", from.id());
        } else {
            return Err(anyhow!("invalid results from revparse"));
        }

        Ok(revspec.from().unwrap().id().to_string())
    }
}

fn fetch<'a>(
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
    repo.reference_to_annotated_commit(&fetch_head)
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
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;

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

fn merge<'a>(
    repo: &'a Repository,
    remote_branch: &str,
    fetch_commit: git2::AnnotatedCommit<'a>,
) -> Result<(), git2::Error> {
    let (analysis, _preference) = repo.merge_analysis(&[&fetch_commit])?;

    if analysis.is_fast_forward() {
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
    } else if analysis.is_normal() {
        let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
        normal_merge(repo, &head_commit, &fetch_commit)?;
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
            Path::new(&format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())),
            None,
        )
    });

    callbacks
}
