#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("network error: {cause}")]
    Network { cause: String },
    #[error("response parsing error: {cause}")]
    Parse { cause: String },
}
