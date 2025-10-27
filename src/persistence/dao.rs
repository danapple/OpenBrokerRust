use deadpool_postgres::{Object, Pool, Transaction};
use log::error;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum DaoError {
    PoolFailed { description: String },
    BeginFailed { description: String },
    CommitFailed { description: String },
    RollbackFailed { description: String } ,
    ExecuteFailed { description: String },
    QueryFailed { description: String },
    OptimisticLockingFailed { description: String },
}

impl fmt::Display for DaoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DaoError::PoolFailed { ref description } => description.fmt(f),
            DaoError::BeginFailed { ref description } => description.fmt(f),
            DaoError::CommitFailed { ref description } => description.fmt(f),
            DaoError::RollbackFailed { ref description } => description.fmt(f),
            DaoError::ExecuteFailed { ref description } => description.fmt(f),
            DaoError::QueryFailed { ref description } => description.fmt(f),
            DaoError::OptimisticLockingFailed { ref description } => description.fmt(f),
        }
    }
}

impl Error for DaoError {
    fn description(&self) -> &str {
        match *self {
            DaoError::PoolFailed { ref description } => description,
            DaoError::BeginFailed { ref description } => description,
            DaoError::CommitFailed { ref description } => description,
            DaoError::RollbackFailed { ref description } => description,
            DaoError::ExecuteFailed { ref description } => description,
            DaoError::QueryFailed { ref description } => description,
            DaoError::OptimisticLockingFailed { ref description } => description,
        }
    }
}

pub fn gen_dao_error(method: &str, y: tokio_postgres::Error) -> DaoError {
    error!("{} {}: {}", method, y.to_string(), match y.as_db_error() {
                Some(x) => format!("{}", x),
                None => "none".to_string()});
    DaoError::ExecuteFailed {
        description: y.to_string()
    }
}

#[derive(Clone)]
pub struct Dao {
    pool: Pool,
}

pub struct DaoTransaction<'a> {
    pub transaction: Transaction<'a>
}

impl Dao {
    pub fn new(pool: Pool) -> Dao {
        Dao {
            pool
        }
    }

    pub async fn get_connection(&self) -> Result<Object, DaoError> {
        match self.pool.get().await {
            Ok(x) => Ok(x),
            Err(pool_error) => {
                Err(DaoError::PoolFailed { description: pool_error.to_string() })
            },
        }
    }

    pub async fn begin<'b> (&self, manager: &'b mut Object) -> Result<DaoTransaction<'b>, DaoError> {
        let txn_builder = manager.build_transaction();
        let start_result = txn_builder.start().await;
        let txn = match start_result {
            Ok(x) => x,
            Err(tx_error) => {
                return Err(DaoError::BeginFailed { description: tx_error.to_string() })
            },
        };
        Ok(DaoTransaction {
            transaction: txn
        })
    }
}



impl<'b> DaoTransaction<'b> {
    pub async fn commit(self) -> Result<(), DaoError> {
        match self.transaction.commit().await {
            Ok(_) => Ok(()),
            Err(transaction_error) => {
                Err(DaoError::CommitFailed { description: transaction_error.to_string() })
            },
        }
    }

    pub async fn rollback(self) -> Result<(), DaoError> {
        match self.transaction.rollback().await {
            Ok(_) => Ok(()),
            Err(transaction_error) => {
                Err(DaoError::RollbackFailed { description: transaction_error.to_string() })
            },
        }
    }
}
