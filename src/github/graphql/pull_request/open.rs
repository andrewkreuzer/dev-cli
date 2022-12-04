use anyhow::anyhow;
use cynic::{http::ReqwestExt, MutationBuilder};

use crate::github::client::GithubClient;
use queries::{PullRequest, PullRequestOpen, PullRequestOpenArguments};

pub async fn run_query(
    client: &GithubClient,
    variables: PullRequestOpenArguments,
) -> Result<PullRequest, anyhow::Error> {
    let query = build_query(variables);

    let res = client.post().run_graphql(query).await?;

    let errors = res.errors.as_ref().and_then(|errors| {
        errors
            .iter()
            .map(|e| e.message.as_str())
            .collect::<Vec<&str>>()
            .join("\n")
            .into()
    });

    if let Some(errors) = errors {
        return Err(anyhow!(errors));
    }

    res.data
        .and_then(|d| d.create_pull_request)
        .and_then(|c| c.pull_request)
        .ok_or(anyhow!("no pull request returned from query"))
}

fn build_query(
    variables: PullRequestOpenArguments,
) -> cynic::Operation<PullRequestOpen, PullRequestOpenArguments> {
    PullRequestOpen::build(variables)
}

#[cynic::schema_for_derives(file = "graphql/github.graphql", module = "schema")]
pub mod queries {
    use crate::github::graphql::schema;

    #[derive(cynic::QueryVariables, Debug)]
    pub struct PullRequestOpenArguments {
        pub base_ref: String,
        pub head_ref: String,
        pub pr_title: String,
        pub repo_id: cynic::Id,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Mutation", variables = "PullRequestOpenArguments")]
    pub struct PullRequestOpen {
        #[arguments(input: {
            baseRefName: $base_ref,
            headRefName: $head_ref,
            repositoryId: $repo_id,
            title: $pr_title
        })]
        pub create_pull_request: Option<CreatePullRequestPayload>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct CreatePullRequestPayload {
        pub client_mutation_id: Option<String>,
        pub pull_request: Option<PullRequest>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct PullRequest {
        pub base_ref_name: String,
        pub head_ref_name: String,
        pub number: i32,
        pub title: String,
    }
}
