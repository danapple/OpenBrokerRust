use crate::entities::account::Account;
use crate::persistence::dao::{gen_dao_error, DaoError, DaoTransaction};
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
        let account = convert_row_to_account(row);

        Ok(account)
    }

    pub async fn get_account(&self, account_id: i64) -> Result<Account, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(ACCOUNT_QUERY);
        query_string.push_str(" WHERE account.accountId = $1");
        let row = match self.transaction.query_one(&query_string,
                                                   &[&account_id]).await {
            Ok(x) => x,
            Err(db_error) => { return Err(gen_dao_error("get_account", db_error)); }

        };
        let account = convert_row_to_account(row);

        Ok(account)
    }

}

fn convert_row_to_account(row: Row) -> Account {
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
