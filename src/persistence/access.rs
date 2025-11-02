use crate::dtos::account::Privilege;
use crate::entities::account::Access;
use crate::persistence::dao::{gen_dao_error, DaoError, DaoTransaction};
use std::str::FromStr;
use tokio_postgres::Row;

impl<'b> DaoTransaction<'b> {
    pub async fn get_accesses_for_actor(&self, actor_id: i32) -> Result<Vec<Access>, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(ACCESS_QUERY);
        query_string.push_str("WHERE actor.actorId = $1 ");
        let res = match self.transaction.query(&query_string,
                                               &[
                                                   &actor_id
                                               ]).await {
            Ok(res) => res,
            Err(db_error) => { return Err(gen_dao_error("get_accesses_for_actor", db_error)); }
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
            actor_id: row.get("actorId"),
            account_id: row.get("accountId"),
            nickname: row.get("nickname"),
            privilege,
        })
    }
}

const ACCESS_QUERY: &str = "\
SELECT actor.actorId, account.accountId, relation.nickname, access.privilege \
FROM actor \
JOIN actor_account_relationship relation on relation.actorId = actor.actorId \
JOIN account on account.accountId = relation.accountId \
JOIN access on access.relationshipId = relation.relationshipId \
";
