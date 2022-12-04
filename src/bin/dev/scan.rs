use std::{path::PathBuf, env};

use anyhow::bail;
use log::info;
use regex::Regex;

use dev_cli::{git::{self, GitRepository}, config};

pub async fn handle_scan_command(
    directory: Option<PathBuf>,
    recurse: bool,
    add: bool,
    config: &mut config::Config
) -> Result<(), anyhow::Error> {
    let cwd = env::current_dir()
        .expect("error getting current directory");

    let directory = match directory {
        Some(dir) => dir,
        None => cwd.clone()
    };

    for (path, repo) in git::scan(&directory, recurse)?.into_iter() {

        // default to origin remote for now
        let mut url = None;
        if let Ok(origin) = repo.find_remote("origin") {
            url = match origin.url() {
                Some(url) => Some(url.to_string()),
                None => None
            };
        }

        let dir = path
            .file_name()
            .and_then(|d| d.to_str())
            .and_then(|d| Some(d.to_string()));

        let relativised_path = match cwd.join(&path).strip_prefix(&cwd) {
            Ok(p) => {
                if cwd.join(p) == cwd {
                    Some(".".to_string())
                } else {
                    Some(p.to_str().unwrap().to_string())
                }
            }
            Err(e) => bail!(e),
        };


        let (name, org) = match &url {
            Some(url) => parse_remote_url(url),
            None => {
                (dir.clone().unwrap(), None)
            }
        };

        if add {
            let git_repo = GitRepository{
                name: name.clone(),
                org,
                url,
                path: relativised_path,
            };
            if git_repo.eq(config.get_root()) {
                info!("{name} is root, skipping");
                continue
            }
            config.add_repo(Some(name), &git_repo)?;

            config.update()?;
        }
    }

    Ok(())
}

fn parse_remote_url(url: &str) -> (String, Option<String>) {
    let re = Regex::new(
        r"(https|git)(://)?(@?)(\w+).com(:|/)(\w+)/([\w-]+).git"
    ).unwrap();
    let caps = re.captures(&url).unwrap();
    let org = caps.get(6).unwrap().as_str();
    let name = caps.get(7).unwrap().as_str();

    (name.to_string(), Some(org.to_string()))
}
