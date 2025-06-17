use anyhow::{anyhow, Context};
use cynic::{GraphQlError, GraphQlResponse};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphQlResponseError {
    #[error("GraphQL execution error: {0}")]
    ExecutionError(String),
    #[error("GraphQL network error")]
    NetworkError(#[from] reqwest::Error),
    #[error("GraphQL missing data: {0}")]
    MissingData(String),
}

/// Process a GraphQL response, extracting data or providing detailed error information
pub fn process_response<T>(
    response: GraphQlResponse<T>,
    context: &str,
) -> Result<T, anyhow::Error> {
    // Check for GraphQL errors first
    if let Some(errors) = response.errors {
        if !errors.is_empty() {
            let error_messages = format_graphql_errors(&errors);
            return Err(GraphQlResponseError::ExecutionError(error_messages).into());
        }
    }

    // Try to extract data
    response
        .data
        .ok_or_else(|| GraphQlResponseError::MissingData(context.to_string()).into())
}

/// Format GraphQL errors into a readable string
pub fn format_graphql_errors(errors: &[GraphQlError]) -> String {
    errors
        .iter()
        .map(|e| {
            let location = e
                .locations
                .as_ref()
                .map(|locs| {
                    locs.iter()
                        .map(|loc| format!("line {} column {}", loc.line, loc.column))
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_else(|| "unknown location".to_string());

            let path = e
                .path
                .as_ref()
                .map(|path| format!(" (path: {})", path.join(".")))
                .unwrap_or_default();

            format!("{} at {}{}", e.message, location, path)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Helper trait to provide a context method for Option<T> that returns a Result
pub trait OptionExt<T> {
    fn context_err(self, context: &str) -> Result<T, anyhow::Error>;
}

impl<T> OptionExt<T> for Option<T> {
    fn context_err(self, context: &str) -> Result<T, anyhow::Error> {
        self.ok_or_else(|| anyhow!("{}", context))
            .with_context(|| format!("Missing data: {}", context))
    }
}