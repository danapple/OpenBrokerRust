#[derive(Debug)]
pub enum BrokerError {
    Failure { description: String },
}