use log::error;

pub mod git;
pub mod github;
pub mod clap;
pub mod repo;
pub mod repos;
pub mod scan;

#[tokio::main]
async fn main() {
    if let Err(e) = clap::init().await {
        error!("{e}")
    }
}
