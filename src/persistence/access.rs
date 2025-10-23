use crate::access_control::Privilege;
use crate::persistence::dao::{DaoError, DaoTransaction};
use log::{error, info};

impl<'b> DaoTransaction<'b> {
    pub async fn is_allowed(&self, account_key: &str, customer_key: &str, privilege: Privilege) -> Result<bool, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(ACCESS_QUERY);
        let res = match self.transaction.query(&query_string,
                                               &[
                                                   &customer_key,
                                                   &account_key,
                                                   &privilege.to_string()]).await {
            Ok(x) => x,
            Err(y) => { error!("is_allowed {}: {}", y.to_string(), match y.as_db_error() {Some(x) => format!("{}", x),None => "none".parse().unwrap()}); return Err(DaoError::ExecuteFailed { description: y.to_string() })},

        };
        info!("res.is_empty() = {}", res.is_empty());
        Ok(!res.is_empty())
    }
}

const ACCESS_QUERY: &str = "\
SELECT customer.customerId, account.accountId, relation.nickname, access.privilege \
FROM customer \
JOIN customer_account_relationship relation on relation.customerId = customer.customerId \
JOIN account on account.accountId = relation.accountId \
JOIN api_key on api_key.customerId = customer.customerId \
JOIN access on access.relationshipId = relation.relationshipId \
WHERE api_key.apiKey = $1 \
and account.accountKey = $2 \
and access.privilege = $3 \
";
