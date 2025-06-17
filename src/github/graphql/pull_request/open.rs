use anyhow::anyhow;
use cynic::{http::ReqwestExt, MutationBuilder};

use crate::github::{
    client::GithubClient,
    graphql::errors::{process_response, OptionExt},
};
use queries::{PullRequest, PullRequestOpen, PullRequestOpenArguments};

pub async fn run_query(
    client: &GithubClient,
    variables: PullRequestOpenArguments,
) -> Result<PullRequest, anyhow::Error> {
    let query = build_query(variables);

    let response = client.post().run_graphql(query).await?;
    
    // Process the GraphQL response to extract data or get detailed error info
    let data = process_response(response, "PullRequest mutation")?;
    
    // Extract the pull request data from the response with descriptive error messages
    data.create_pull_request
        .context_err("Failed to create pull request")?
        .pull_request
        .context_err("Pull request data missing in response")
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
