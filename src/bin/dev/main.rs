use std::process::exit;

use log::error;

mod clap;
mod init;
mod git;
mod github;
mod repo;
mod run;
mod scan;
mod shell;
mod yaml;

#[tokio::main]
async fn main() {
    if let Err(e) = clap::init().await {
        error!("{:?}", e);
        exit(1);
    }
}
