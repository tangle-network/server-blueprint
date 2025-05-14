#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// SDK error
    #[error(transparent)]
    Sdk(#[from] blueprint_sdk::error::Error),

    #[error("Service {0} no longer exists")]
    ServiceNotFound(u64),
    #[error("Missing request params")]
    MissingRequestParams,
    #[error("Invalid request params: {0}")]
    InvalidRequestParams(#[from] blueprint_sdk::tangle::serde::error::Error),
    #[error("Invalid request params: unknown runtime")]
    UnknownRuntime,
    #[error("Missing port binding")]
    MissingPortBinding,

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
