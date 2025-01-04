use std::error::Error;

use crate::Rule;

#[derive(Debug, thiserror::Error)]
pub enum ArmParserError {
    #[error("Pest error: {0}")]
    PestError(#[from] pest::error::Error<Rule>),
    #[error("ParseInt error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
}

pub trait PrintError {
    fn print_error(self, loc: &str) -> Self;
}

impl<T, E> PrintError for Result<T, E>
where
    E: Error,
{
    fn print_error(self, loc: &str) -> Self {
        self.inspect_err(|e| {
            eprintln!("Error at {}: {}", loc, e);
        })
    }
}
