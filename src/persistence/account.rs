use crate::entities::account::Account;
use crate::persistence::dao::{gen_dao_error, DaoError, DaoTransaction};
use std::collections::HashMap;
use tokio_postgres::Row;

impl<'b> DaoTransaction<'b> {
    pub async fn get_account_by_account_key(&self, account_key: &String) -> Result<Account, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(ACCOUNT_QUERY);
        query_string.push_str(" WHERE account.accountKey = $1");
        let row = match self.transaction.query_one(&query_string,
                                               &[&account_key]).await {
            Ok(x) => x,
            Err(db_error) => { return Err(gen_dao_error("get_account_by_account_key", db_error)); }

        };
        let account = convert_row_to_account(&row);
        Ok(account)
    }

    pub async fn get_account(&self, account_id: i32) -> Result<Option<Account>, DaoError> {
        let accounts_map = match self.get_accounts(vec![account_id]).await {
            Ok(accounts_map) => accounts_map,
            Err(db_error) => return Err(db_error)
        };
        Ok(match accounts_map.get(&account_id) {
            Some(account) => Some(account.clone()),
            None => None
        })
    }

    pub async fn get_accounts(&self, account_ids: Vec<i32>) -> Result<HashMap<i32, Account>, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(ACCOUNT_QUERY);
        query_string.push_str(" WHERE account.accountId = ANY ($1)");
        let rows = match self.transaction.query(&query_string,
                                                   &[&account_ids]).await {
            Ok(x) => x,
            Err(db_error) => { return Err(gen_dao_error("get_accounts", db_error)); }
        };

        let accounts_vec = rows.iter().map(|row| convert_row_to_account(row)).collect::<Vec<Account>>();
        let accounts_map = accounts_vec.iter().map(|account| (account.account_id, account.clone())).collect();

        Ok(accounts_map)
    }
}

fn convert_row_to_account(row: &Row) -> Account {
    Account {
        account_id: row.get("accountId"),
        account_key: row.get("accountKey"),
        account_number: row.get("accountNumber"),
        account_name: row.get("accountName"),
    }
}

const ACCOUNT_QUERY: &str = "\
SELECT accountId, accountKey, accountNumber, accountName \
FROM account \
";
