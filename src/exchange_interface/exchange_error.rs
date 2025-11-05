use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ExchangeError {
    Failure {
        description: String,
        cause: String
    },
}

impl ExchangeError {
    pub fn failure(description: String, 
                   cause: String) -> Self {
        ExchangeError::Failure{
            description,
            cause,
        }
    }
}

impl fmt::Display for ExchangeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ExchangeError::Failure {
                ref description ,
                ref cause,
            } => write!(f, "{}: cause {}", description, cause),
        }
    }
}

impl Error for ExchangeError {}