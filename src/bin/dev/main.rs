use log::error;

pub mod clap;
pub mod git;
pub mod github;
pub mod repo;
pub mod repos;
pub mod scan;

#[tokio::main]
async fn main() {
    if let Err(e) = clap::init().await {
        error!("{e}")
    }
}
