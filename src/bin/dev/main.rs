use std::process::exit;
use log::error;

#[tokio::main]
async fn main() {
    if let Err(e) = clap::init().await {
        error!("{:?}", e);
        exit(1);
    }
}

mod clap;
mod git;
mod github;
mod init;
mod repo;
mod run;
mod scan;
mod shell;
mod yaml;
