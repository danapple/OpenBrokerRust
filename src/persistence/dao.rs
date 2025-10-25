use crate::entities::account::Position;
use crate::entities::trading::{Order, OrderState};
use crate::rest_api::trading::OrderStatus;
use deadpool_postgres::{Object, Pool, Transaction};
use log::error;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use tokio_postgres::Row;

#[derive(Debug)]
pub enum DaoError {
    CommitFailed { description: String },
    RollbackFailed { description: String } ,
    ExecuteFailed { description: String },
    QueryFailed { description: String },
    OptimisticLockingFailed { description: String },
}

impl fmt::Display for DaoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
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
            DaoError::CommitFailed { ref description } => description,
            DaoError::RollbackFailed { ref description } => description,
            DaoError::ExecuteFailed { ref description } => description,
            DaoError::QueryFailed { ref description } => description,
            DaoError::OptimisticLockingFailed { ref description } => description,
        }
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

    pub async fn get_connection(&self) -> Object {
        match self.pool.get().await {
            Ok(x) => x,
            Err(y) => {error!("make_manager {}", y); todo!()},
        }
    }

    pub async fn begin<'b> (&self, manager: &'b mut Object) -> DaoTransaction<'b> {
        let txn_builder = manager.build_transaction();
        let start_result = txn_builder.start().await;
        let txn = match start_result {
            Ok(x) => x,
            Err(y) => {error!("begin {}", y); todo!()},
        };
        DaoTransaction {
            transaction: txn
        }
    }
}



impl<'b> DaoTransaction<'b> {
    pub async fn commit(self) -> Result<(), DaoError> {
        match self.transaction.commit().await {
            Ok(_) => Ok(()),
            Err(y) => {
                error!("begin {}", y);
                Err(DaoError::CommitFailed { description: y.to_string() })
            },
        }
    }

    pub async fn rollback(self) -> Result<(), DaoError> {
        match self.transaction.rollback().await {
            Ok(_) => Ok(()),
            Err(y) => {
                error!("rollback {}", y);
                Err(DaoError::RollbackFailed { description: y.to_string() })
            },
        }
    }
}
