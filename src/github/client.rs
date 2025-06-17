use ::reqwest::{Client, RequestBuilder};
use anyhow::{anyhow, Context};
use thiserror::Error;

use super::graphql::{
    pull_request::open::{
        queries::{PullRequest, PullRequestOpenArguments},
        run_query,
    },
    errors::OptionExt,
};

#[derive(Error, Debug)]
pub enum GithubClientError {
    #[error("GitHub authentication error: {0}")]
    AuthenticationError(String),
    #[error("GitHub API request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("GitHub environment error: {0}")]
    EnvironmentError(String),
}

impl From<std::env::VarError> for GithubClientError {
    fn from(err: std::env::VarError) -> Self {
        GithubClientError::EnvironmentError(format!("Missing environment variable: {}", err))
    }
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn open_pr(
    base_ref: &str,
    head_ref: &str,
    pr_title: &str,
    repo_id: &str,
) -> Result<PullRequest, anyhow::Error> {
    let client = GithubClient::new()?;
    let pr_args = PullRequestOpenArguments {
        base_ref: base_ref.to_string(),
        head_ref: head_ref.to_string(),
        pr_title: pr_title.to_string(),
        repo_id: cynic::Id::new(repo_id),
    };

    let pull_request = run_query(&client, pr_args).await?;
    Ok(pull_request)
}

pub struct GithubClient {
    endpoint: String,
    client: Client,
}

impl GithubClient {
    fn new() -> Result<Self, anyhow::Error> {
        // Get GitHub token with better error message
        let github_token = std::env::var("GITHUB_TOKEN")
            .map_err(GithubClientError::from)
            .with_context(|| "GitHub token is required. Set the GITHUB_TOKEN environment variable.")?;

        // Validate token is not empty
        if github_token.trim().is_empty() {
            return Err(GithubClientError::AuthenticationError(
                "GitHub token is empty".to_string(),
            )
            .into());
        }

        // Build the client with error handling
        let client = Client::builder()
            .user_agent(format!("dev-cli/v{VERSION}"))
            .default_headers(
                std::iter::once((
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(
                        format!("Bearer {}", github_token).as_str(),
                    )
                    .map_err(|e| anyhow!("Invalid header value: {}", e))?,
                ))
                .collect(),
            )
            .build()
            .map_err(GithubClientError::from)
            .context("Failed to build HTTP client")?;

        Ok(GithubClient {
            endpoint: "https://api.github.com/graphql".to_string(),
            client,
        })
    }

    pub fn post(&self) -> RequestBuilder {
        self.client.post(&self.endpoint)
    }
}
