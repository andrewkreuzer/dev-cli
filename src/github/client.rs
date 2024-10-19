use ::reqwest::{Client, RequestBuilder};
use anyhow::anyhow;

use super::graphql::pull_request::open::{
    queries::{PullRequest, PullRequestOpenArguments},
    run_query,
};

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
        let github_token = match std::env::var("GITHUB_TOKEN") {
            Ok(token) => token,
            Err(err) => return Err(anyhow!("{err}: \"GITHUB_TOKEN\"")),
        };

        let client = Client::builder()
            .user_agent(format!("dev-cli/v{VERSION}"))
            .default_headers(
                std::iter::once((
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(
                        format!("Bearer {}", github_token).as_str(),
                    )?,
                ))
                .collect(),
            )
            .build()?;

        Ok(GithubClient {
            endpoint: "https://api.github.com/graphql".to_string(),
            client,
        })
    }

    pub fn post(&self) -> RequestBuilder {
        self.client.post(&self.endpoint)
    }
}
