use crate::entities::account::Balance;
use crate::persistence::dao::{gen_dao_error, DaoError, DaoTransaction};
use log::error;
use tokio_postgres::Row;

impl<'b> DaoTransaction<'b> {
    pub async fn update_balance(&self, balance: &mut Balance) -> Result<(), DaoError> {
        let next_version_number = balance.version_number + 1;
        let rows_updated = match self.transaction.execute(BALANCE_UPDATE_STATEMENT,
                                                          &[
                                                              &balance.cash,
                                                              &balance.update_time,
                                                              &next_version_number,
                                                              &balance.balance_id,
                                                              &balance.version_number,
                                                          ]
        ).await {
            Ok(rows) => rows,
            Err(db_error) => { return Err(gen_dao_error("update_balance", db_error)); }

        };
        if rows_updated == 0 {
            return Err(DaoError::OptimisticLockingFailed{ description: "update balance 0 rows modified".to_string() });
        }
        balance.version_number = next_version_number;
        Ok(())
    }

    pub async fn get_balance(&self, account_key: &String) -> Result<Balance, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(BALANCE_QUERY);
        query_string.push_str(" WHERE account.accountKey = $1");
        let res = match self.transaction.query(&query_string,
                                               &[&account_key]).await {
            Ok(x) => x,
            Err(db_error) => { return Err(gen_dao_error("get_balance", db_error)); }
        };
        match res.first() {
            Some(row) => {
                Ok(convert_row_to_balance(row))
            },
            None => { error!("get_balance No balance available");
                Err(DaoError::QueryFailed{ description: "No row available".to_string() })},
        }
    }

}


fn convert_row_to_balance(row: &Row) -> Balance {
    Balance {
        balance_id: row.get("balanceId"),
        account_id: row.get("accountId"),
        cash: row.get("cash"),
        update_time: row.get("updateTime"),
        version_number: row.get("versionNumber"),
    }
}

const BALANCE_QUERY: &str = "
SELECT balanceId, balance.accountId, cash, updateTime, versionNumber FROM balance \
JOIN account on account.accountId = balance.accountId \
";

const BALANCE_UPDATE_STATEMENT: &str = "
UPDATE balance \
set cash = $1, updateTime = $2, versionNumber = $3 \
WHERE balanceId = $4 and versionNumber = $5 \
";

