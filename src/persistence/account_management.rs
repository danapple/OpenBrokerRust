use crate::entities::actor::Actor;
use crate::persistence::dao::{gen_dao_error, DaoError, DaoTransaction};
use crate::time::current_time_millis;
use log::debug;
use rand::Rng;
use uuid::Uuid;
// 0.8.5

impl<'b> DaoTransaction<'b> {
    pub async fn create_account_for_actor(&self, actor: &Actor) -> Result<(), DaoError> {

        let mut account_number_option = None;
        for iteration in 1..101 {
            let account_number = rand::rng().random_range(100000..999999);
            debug!("Trying account number {}, iteration {}", account_number, iteration);
            let rows = match self.transaction.query("SELECT accountNumber FROM account WHERE accountNumber = $1",
                                             &[&(account_number).to_string()]).await {
                Ok(rows) => rows,
                Err(db_error) => { return Err(gen_dao_error("create_account_for_actor test account number", db_error)); }
            };
            if rows.len() == 0 {
                account_number_option = Some(account_number.to_string());
                break
            }
        };

        let account_number = match account_number_option {
            Some(account_number) => account_number,
            None =>  return Err(DaoError::ConversionFailed { description: "Could not generate account number".to_string() })

        };

        let row = match self.transaction.query_one(
            "INSERT INTO account \
            (accountKey, accountNumber, accountName) \
            VALUES ($1, $2, $3) \
            RETURNING accountId",
            &[&Uuid::new_v4().simple().to_string(),
                &account_number,
                &format!("{} initial account", actor.actor_name),
            ]
        ).await {
            Ok(x) => x,
            Err(db_error) => { return Err(gen_dao_error("create_account_for_actor account", db_error)); }
        };
        let account_id: i32 = row.get("accountId");

        match self.transaction.execute(
            "INSERT INTO balance \
            (accountId, cash, updateTime, versionNumber) \
            VALUES ($1, $2, $3, $4) \
            ",
            &[&account_id,
                &100000f32,
                &current_time_millis(),
                &0i64
            ]
        ).await {
            Ok(x) => x,
            Err(db_error) => { return Err(gen_dao_error("create_account_for_actor balance", db_error)); }
        };

        let row = match self.transaction.query_one(
            "INSERT INTO actor_account_relationship \
            (actorId, accountId, nickname) \
            VALUES ($1, $2, $3) \
            RETURNING relationshipId",
            &[
                &actor.actor_id,
                &account_id,
                &format!("{} nickname", account_number),
            ]
        ).await {
            Ok(x) => x,
            Err(db_error) => { return Err(gen_dao_error("create_account_for_actor actor_account_relationship", db_error)); }
        };
        let relationship_id: i32 = row.get("relationshipId");

        match self.transaction.execute(
            "INSERT INTO access \
            (relationshipId, privilege) \
            VALUES ($1, 'Owner'), \
             ($1, 'Read'), \
             ($1, 'Submit'), \
             ($1, 'Cancel') ",
            &[&relationship_id
            ]
        ).await {
            Ok(_) => {},
            Err(db_error) => { return Err(gen_dao_error("create_account_for_actor access", db_error)); }
        };

        match self.transaction.execute(
            "INSERT INTO api_key \
            (actorId, apiKey) \
            VALUES ($1, $2) \
            ",
            &[
                &actor.actor_id,
                &&Uuid::new_v4().simple().to_string(),
            ]
        ).await {
            Ok(x) => x,
            Err(db_error) => { return Err(gen_dao_error("create_account_for_actor api_key", db_error)); }
        };

        Ok(())
    }
}