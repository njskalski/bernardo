use thiserror::Error;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Unsupported query type (in given context): {details}")]
    UnsupporedQueryType { details: &'static str },
}
