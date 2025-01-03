use crate::Rule;

#[derive(Debug, thiserror::Error)]
pub enum ArmParserError {
    #[error("Pest error: {0}")]
    PestError(#[from] pest::error::Error<Rule>),
    #[error("ParseInt error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
}
