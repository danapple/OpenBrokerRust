use crate::entities::account::Access;
use crate::persistence::dao::{gen_dao_error, DaoError, DaoTransaction};
use crate::rest_api::account::Privilege;
use std::str::FromStr;
use tokio_postgres::Row;

impl<'b> DaoTransaction<'b> {
    pub async fn get_accesses_for_customer(&self, customer_id: i64) -> Result<Vec<Access>, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(ACCESS_QUERY);
        query_string.push_str("WHERE customer.customerId = $1 ");
        let res = match self.transaction.query(&query_string,
                                               &[
                                                   &customer_id
                                               ]).await {
            Ok(res) => res,
            Err(db_error) => { return Err(gen_dao_error("get_accesses_for_customer", db_error)); }
        };
        let mut accesses = Vec::new();
        for row in res {
            let access = match self.convert_row_to_access(&row) {
                Ok(access) => access,
                Err(dao_error) => return Err(dao_error)
            };
            accesses.push(access);
        }
        Ok(accesses)
    }

    fn convert_row_to_access(&self, row: &Row) -> Result<Access, DaoError> {
        let row_privilege = row.get("privilege");

        let privilege_result = Privilege::from_str(row_privilege);
        let privilege = match privilege_result {
            Ok(privilege) => privilege,
            Err(()) => {
                return Err(DaoError::ConversionFailed {
                    description: format!("Unknown order status {}", row_privilege)
                })
            }
        };
        Ok(Access {
            customer_id: row.get("customerId"),
            account_id: row.get("accountId"),
            nickname: row.get("nickname"),
            privilege,
        })
    }
}

const ACCESS_QUERY: &str = "\
SELECT customer.customerId, account.accountId, relation.nickname, access.privilege \
FROM customer \
JOIN customer_account_relationship relation on relation.customerId = customer.customerId \
JOIN account on account.accountId = relation.accountId \
JOIN api_key on api_key.customerId = customer.customerId \
JOIN access on access.relationshipId = relation.relationshipId \
";
