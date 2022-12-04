use cynic::{http::ReqwestExt, GraphQlResponse, QueryBuilder};

use crate::github::client::GithubClient;
use queries::{RepositoryInfo, RepositoryInfoArguments};

pub async fn run_query(
    client: &GithubClient,
    name: String,
    owner: String,
) -> Result<GraphQlResponse<RepositoryInfo>, anyhow::Error> {
    let query = build_query(name, owner);

    let response_data = client.post().run_graphql(query).await?;

    Ok(response_data)
}

fn build_query(
    name: String,
    owner: String,
) -> cynic::Operation<RepositoryInfo, RepositoryInfoArguments> {
    RepositoryInfo::build(RepositoryInfoArguments { name, owner })
}

#[cynic::schema_for_derives(file = "graphql/github.graphql", module = "schema")]
mod queries {
    use crate::github::graphql::schema;

    #[derive(cynic::QueryVariables, Debug)]
    pub struct RepositoryInfoArguments {
        pub name: String,
        pub owner: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Query", variables = "RepositoryInfoArguments")]
    pub struct RepositoryInfo {
        #[arguments(owner: $owner, name: $name)]
        pub repository: Option<Repository>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Repository {
        pub name: String,
        pub id: cynic::Id,
        #[arguments(refPrefix: "refs/heads/", first: 3)]
        pub refs: Option<RefConnection>,
        #[arguments(first: 3, states: "OPEN")]
        pub pull_requests: PullRequestConnection,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct RefConnection {
        pub nodes: Option<Vec<Option<Ref>>>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Ref {
        pub id: cynic::Id,
        pub name: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct PullRequestConnection {
        pub nodes: Option<Vec<Option<PullRequest>>>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct PullRequest {
        pub id: cynic::Id,
        pub title: String,
        pub author: Option<Actor>,
        pub base_ref_name: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Actor {
        pub login: String,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug)]
    pub enum PullRequestState {
        Closed,
        Merged,
        Open,
    }
}
