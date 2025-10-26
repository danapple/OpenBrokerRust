use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum BrokerError {
    Failure { description: String },
}

impl BrokerError {
    pub fn failure(description: String) -> Self {
        BrokerError::Failure{
            description,
        }
    }
}

impl fmt::Display for BrokerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BrokerError::Failure { ref description } => description.fmt(f),
        }
    }
}

impl Error for BrokerError {}