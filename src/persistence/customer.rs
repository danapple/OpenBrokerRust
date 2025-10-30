use crate::entities::customer::Customer;
use crate::persistence::dao::{gen_dao_error, DaoError, DaoTransaction};
use tokio_postgres::Row;

impl<'b> DaoTransaction<'b> {
    pub async fn check_offer_code(&self, offer_code: &str) -> Result<bool, DaoError> {
        let res = match self.transaction.query("SELECT count(1) FROM offer_code WHERE offerCode = $1",
                                                   &[&offer_code]).await {
            Ok(res) => res,
            Err(db_error) => { return Err(gen_dao_error("check_offer_code", db_error)); }

        };
        Ok(res.len() == 1)
    }

    pub async fn get_customer(&self, email_address: &str) -> Result<Option<Customer>, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(CUSTOMER_QUERY);
        query_string.push_str("WHERE emailAddress = $1");
        let row = match self.transaction.query_one(&query_string,
                                               &[&email_address]).await {
            Ok(res) => res,
            Err(db_error) => { return Err(gen_dao_error("get_customer", db_error)); }
        };

        Ok(Some(convert_row_to_customer(&row)))
    }

    pub async fn get_customer_by_api_key(&self, api_key: &str) -> Result<Option<Customer>, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(CUSTOMER_QUERY);
        query_string.push_str(JOIN_API_KEY);
        query_string.push_str("WHERE apiKey = $1");

        let res = match self.transaction.query(&query_string,
                                                   &[&api_key]).await {
            Ok(res) => res,
            Err(db_error) => { return Err(gen_dao_error("get_customer_by_api_key", db_error)); }
        };
        if res.is_empty() {
            return Ok(None);
        }
        if res.len() != 1 {
            return Err(DaoError::QueryFailed {
                description: format!("get_customer_by_api_key got {} rows, expected 1", res.len()),
            });
        };
        Ok(res.iter().next().map(convert_row_to_customer))
    }

    pub async fn get_customer_password_hash(&self, email_address: &str) -> Result<Option<String>, DaoError> {
        let row = match self.transaction.query_one(CUSTOMER_PASSWORD_HASH_QUERY,
                                                   &[&email_address]).await {
            Ok(res) => res,
            Err(db_error) => { return Err(gen_dao_error("get_customer_password_hash", db_error)); }
        };

        Ok(row.get("passwordHash"))
    }

    pub async fn save_customer(&self, email_address: &str, customer_name: &str, offer_code: &str, password_hash: &str) -> Result<Customer, DaoError> {
        let row = match self.transaction.query_one(
            "INSERT INTO customer \
            (customerName, emailAddress, offerCodeId) \
            VALUES ($1, $2, (select offerCodeId FROM offer_code where offerCode = $3)) \
            RETURNING customerId",
            &[&customer_name,
                &email_address,
                &offer_code,
            ]
        ).await {
            Ok(x) => x,
            Err(db_error) => { return Err(gen_dao_error("save_customer customer", db_error)); }
        };
        let customer_id =  row.get("customerId");
        let row_count = match self.transaction.execute(
            "INSERT INTO login_info \
            (customerId, passwordHash) \
            VALUES ($1, $2)",
            &[&customer_id,
                &password_hash,
            ]
        ).await {
            Ok(row_count) => row_count,
            Err(db_error) => { return Err(gen_dao_error("save_customer login_info", db_error)); }
        };
        if row_count != 1 {
            return Err(DaoError::ExecuteFailed { description: format!("login_info insert returned {} rows, not 1", row_count) });
        }
        Ok(Customer {
            customer_id,
            email_address: email_address.to_string(),
            customer_name: customer_name.to_string(),
            offer_code: Some(offer_code.to_string()),
        })

    }
}

fn convert_row_to_customer(row: &Row) -> Customer {
    let offer_code = match row.try_get("offerCode") {
        Ok(offer_code) => Some(offer_code),
        _ => None
    };
    Customer {
        customer_id: row.get("customerId"),
        email_address: row.get("emailAddress"),
        customer_name: row.get("customerName"),
        offer_code,
    }
}

const CUSTOMER_QUERY: &str = "\
SELECT customer.customerId, customerName, emailAddress, offerCode \
FROM customer \
LEFT JOIN offer_code ON offer_code.offerCodeId = customer.offerCodeId \
";

const JOIN_API_KEY: &str = "\
 JOIN api_key on api_key.customerId = customer.customerId \
";

const CUSTOMER_PASSWORD_HASH_QUERY: &str = "\
SELECT passwordHash \
FROM login_info \
JOIN customer on customer.customerId = login_info.customerId \
WHERE emailAddress = $1 \
";

