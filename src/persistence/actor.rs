use crate::entities::actor::Actor;
use crate::persistence::dao::{gen_dao_error, DaoError, DaoTransaction};
use tokio_postgres::Row;

impl<'b> DaoTransaction<'b> {

    pub async fn get_actor(&self, 
                           email_address: &str) -> Result<Option<Actor>, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(ACTOR_QUERY);
        query_string.push_str("WHERE emailAddress = $1");
        let row = match self.transaction.query_one(&query_string,
                                               &[&email_address]).await {
            Ok(res) => res,
            Err(db_error) => { return Err(gen_dao_error("get_actor", db_error)); }
        };

        Ok(Some(convert_row_to_actor(&row)))
    }

    pub async fn get_actor_by_api_key(&self, 
                                      api_key: &str) -> Result<Option<Actor>, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(ACTOR_QUERY);
        query_string.push_str(JOIN_API_KEY);
        query_string.push_str("WHERE apiKey = $1");

        let res = match self.transaction.query(&query_string,
                                                   &[&api_key]).await {
            Ok(res) => res,
            Err(db_error) => { return Err(gen_dao_error("get_actor_by_api_key", db_error)); }
        };
        if res.is_empty() {
            return Ok(None);
        }
        if res.len() != 1 {
            return Err(DaoError::QueryFailed {
                description: format!("get_actor_by_api_key got {} rows, expected 1", res.len()),
            });
        };
        Ok(res.iter().next().map(convert_row_to_actor))
    }

    pub async fn get_actor_password_hash(&self, 
                                         email_address: &str) -> Result<Option<String>, DaoError> {
        let rows = match self.transaction.query(ACTOR_PASSWORD_HASH_QUERY,
                                                   &[&email_address]).await {
            Ok(res) => res,
            Err(db_error) => { return Err(gen_dao_error("get_actor_password_hash", db_error)); }
        };
        if rows.is_empty() {
            return Ok(None);
        }
        Ok(match rows.first() {
            Some(row) => Some(row.get("passwordHash")),
            None =>  None,
        })
    }

    pub async fn save_actor(&self, 
                            email_address: &str, 
                            actor_name: &str, 
                            offer_code: &str, 
                            password_hash: &str) -> Result<Actor, DaoError> {
        let row = match self.transaction.query_one(
            "INSERT INTO actor \
            (actorName, emailAddress, offerId) \
            VALUES ($1, $2, (select offerId FROM offer where code = $3)) \
            RETURNING actorId",
            &[&actor_name,
                &email_address,
                &offer_code,
            ]
        ).await {
            Ok(x) => x,
            Err(db_error) => { return Err(gen_dao_error("save_actor actor", db_error)); }
        };
        let actor_id =  row.get("actorId");
        let row_count = match self.transaction.execute(
            "INSERT INTO login_info \
            (actorId, passwordHash) \
            VALUES ($1, $2)",
            &[&actor_id,
                &password_hash,
            ]
        ).await {
            Ok(row_count) => row_count,
            Err(db_error) => { return Err(gen_dao_error("save_actor login_info", db_error)); }
        };
        if row_count != 1 {
            return Err(DaoError::ExecuteFailed { description: format!("login_info insert returned {} rows, not 1", row_count) });
        }
        Ok(Actor {
            actor_id,
            email_address: email_address.to_string(),
            actor_name: actor_name.to_string(),
            offer_code: Some(offer_code.to_string()),
        })

    }
}

fn convert_row_to_actor(row: &Row) -> Actor {
    let code = match row.try_get("code") {
        Ok(code) => Some(code),
        _ => None
    };
    Actor {
        actor_id: row.get("actorId"),
        email_address: row.get("emailAddress"),
        actor_name: row.get("actorName"),
        offer_code: code,
    }
}

const ACTOR_QUERY: &str = "\
SELECT actor.actorId, actorName, emailAddress, code \
FROM actor \
LEFT JOIN offer ON offer.offerId = actor.offerId \
";

const JOIN_API_KEY: &str = "\
 JOIN api_key on api_key.actorId = actor.actorId \
";

const ACTOR_PASSWORD_HASH_QUERY: &str = "\
SELECT passwordHash \
FROM login_info \
JOIN actor on actor.actorId = login_info.actorId \
WHERE emailAddress = $1 \
";

