use anyhow::{anyhow, bail};
use cynic::{http::ReqwestExt, QueryBuilder};

use crate::github::client::GithubClient;
use queries::{
    IssueOrder, IssueOrderField, OrderDirection, PullRequest, PullRequestTitles, PullRequestTitlesArguments,
};

pub async fn run_query(
    client: &GithubClient,
) -> Result<Vec<PullRequest>, anyhow::Error> {
    let query = build_query();

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
        bail!(errors);
    }

    res.data
        .and_then(|d| d.repository)
        .and_then(|r| {
            r.pull_requests.nodes
            .into_iter()
            .collect::<Vec<PullRequest>>()
            .into()
        })
        .ok_or(anyhow!("no pull requests returned from query"))
}

fn build_query() -> cynic::Operation<PullRequestTitles, PullRequestTitlesArguments> {
    PullRequestTitles::build(PullRequestTitlesArguments {
        pr_order: IssueOrder {
            direction: OrderDirection::Asc,
            field: IssueOrderField::CreatedAt,
        },
    })
}

#[cynic::schema_for_derives(file = "graphql/github.graphql", module = "schema")]
pub mod queries {
    use crate::github::graphql::schema;
    pub type DateTime = chrono::DateTime<chrono::Utc>;

    #[derive(cynic::QueryVariables, Debug)]
    pub struct PullRequestTitlesArguments {
        pub pr_order: IssueOrder,
    }

    #[derive(cynic::InputObject, Clone, Debug)]
    #[cynic(rename_all = "camelCase")]
    pub struct IssueOrder {
        pub direction: OrderDirection,
        pub field: IssueOrderField,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug)]
    #[cynic(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum OrderDirection {
        Asc,
        Desc,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug)]
    #[cynic(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum IssueOrderField {
        Comments,
        CreatedAt,
        UpdatedAt,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Query", variables = "PullRequestTitlesArguments")]
    pub struct PullRequestTitles {
        #[arguments(name = "linode".to_string(), owner = "andrewkreuzer".to_string())]
        pub repository: Option<Repository>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(variables = "PullRequestTitlesArguments")]
    pub struct Repository {
        #[arguments(orderBy: $pr_order, first: 10)]
        pub pull_requests: PullRequestConnection,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct PullRequestConnection {
        #[cynic(flatten)]
        pub nodes: Vec<PullRequest>,
    }

    #[derive(cynic::QueryFragment, Clone, Debug)]
    pub struct PullRequest {
        pub title: String,
        pub created_at: DateTime,
    }
}
