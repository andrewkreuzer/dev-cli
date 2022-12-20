use log::{info, trace};
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use git2::Repository;
use walkdir::{DirEntry, WalkDir};

use super::GitError;

pub fn run(
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

fn scan_directory(directory: &Path) -> Result<(PathBuf, Repository), anyhow::Error> {
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
