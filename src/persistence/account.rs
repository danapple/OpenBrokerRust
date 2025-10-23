use crate::entities::account::{Account, Balance, Position};
use crate::persistence::dao::{DaoError, DaoTransaction};
use log::error;
use tokio_postgres::Row;

impl<'b> DaoTransaction<'b> {

    pub async fn get_account_by_account_key(&self, account_key: &String) -> Result<Account, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(ACCOUNT_QUERY);
        query_string.push_str(" WHERE account.accountKey = $1");
        let row = match self.transaction.query_one(&query_string,
                                               &[&account_key]).await {
            Ok(x) => x,
            Err(y) => { error!("get_account_by_account_key {}", y); return Err(DaoError::QueryFailed{ description: y.to_string() })},
        };
        let account = convert_row_to_accounts(row);

        Ok(account)
    }

    pub async fn get_account(&self, account_id: i64) -> Result<Account, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(ACCOUNT_QUERY);
        query_string.push_str(" WHERE account.accountId = $1");
        let row = match self.transaction.query_one(&query_string,
                                                   &[&account_id]).await {
            Ok(x) => x,
            Err(y) => { error!("get_account_by_account_key {}", y); return Err(DaoError::QueryFailed{ description: y.to_string() })},
        };
        let account = convert_row_to_accounts(row);

        Ok(account)
    }

    pub async fn get_positions(&self, account_key: &String) -> Result<Vec<Position>, DaoError> {

        todo!()
    }

    pub async fn get_balances(&self, account_key: &String) -> Result<Balance, DaoError> {
        todo!()
    }
}

fn convert_row_to_accounts(row: Row) -> Account {
    Account {
        account_id: row.get("accountId"),
        account_key: row.get("accountKey"),
        account_number: row.get("accountNumber"),
        account_name: row.get("accountName"),
    }
}

const ACCOUNT_QUERY: &str = "SELECT accountId, accountKey, accountNumber, accountName \
FROM account \
";