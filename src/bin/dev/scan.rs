use std::{env, path::Path, path::PathBuf};

use anyhow::bail;
use regex::Regex;

use dev_cli::{
    config,
    git::{self, GitRepository},
};

pub async fn handle_scan_command(
    directory: Option<PathBuf>,
    depth: usize,
    recurse: bool,
    add: bool,
    config: &mut config::Config,
) -> Result<(), anyhow::Error> {
    let cwd = env::current_dir().expect("error getting current directory");

    let directory = match directory {
        Some(dir) => dir,
        None => cwd.clone(),
    };

    for (path, repo) in git::scan(&directory, depth, recurse)?.into_iter() {
        // default to origin remote for now
        let mut url = None;
        if let Ok(origin) = repo.find_remote("origin") {
            url = match origin.url() {
                Some(url) => Some(url.to_string()),
                None => None,
            };
        }

        let dir = path
            .file_name()
            .and_then(|d| d.to_str())
            .and_then(|d| Some(d.to_string()))
            .unwrap();

        // TODO: this is bad, should be no reason to create the full path
        let file_full_path = cwd.join(&path);
        let relativised_path = match file_full_path.strip_prefix(&cwd) {
            Ok(p) => is_root_repo(&p, &file_full_path, &cwd),
            Err(e) => bail!(e),
        };

        let (name, org) = match &url {
            Some(url) => parse_remote_url(url),
            None => (dir, None),
        };

        if add {
            let git_repo = GitRepository {
                name: name.clone(),
                org,
                url,
                path: relativised_path,
            };
            config.add_repo(Some(name), &git_repo)?;

            config.update()?;
        }
    }

    Ok(())
}

fn is_root_repo(p: &Path, file_path: &PathBuf, cwd: &PathBuf) -> Option<String> {
    if file_path == cwd {
        Some(".".to_string())
    } else {
        p.to_str().and_then(|p| Some(p.to_string()))
    }
}

fn parse_remote_url(url: &str) -> (String, Option<String>) {
    let re = Regex::new(r"(https|git)(://)?(@?)(\w+).com(:|/)(\w+)/([\w-]+)(.git)?").unwrap();
    let caps = re.captures(&url).unwrap();
    let org = caps.get(6).unwrap().as_str();
    let name = caps.get(7).unwrap().as_str();

    (name.to_string(), Some(org.to_string()))
}
