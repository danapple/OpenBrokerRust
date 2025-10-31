use crate::entities::offer::Offer;
use crate::persistence::dao::{gen_dao_error, DaoError, DaoTransaction};
use crate::time::current_time_millis;

impl<'b> DaoTransaction<'b> {
    pub async fn check_offer(&self, offer_code: &str) -> Result<bool, DaoError> {
        let res = match self.transaction.query("SELECT code FROM offer \
                                        WHERE code = $1 AND \
                                        expirationTime > $2",
                                               &[
                                                   &offer_code,
                                                   &current_time_millis()
                                               ]).await {
            Ok(res) => res,
            Err(db_error) => { return Err(gen_dao_error("check_offer", db_error)); }
        };
        Ok(res.len() == 1)
    }


    pub async fn save_offer(&self, mut offer: Offer) -> Result<(), DaoError> {
        let row = match self.transaction.query_one(
            "INSERT INTO offer \
            (code, description, expirationTime) \
            VALUES ($1, $2, $3) \
            RETURNING offerId",
            &[&offer.code,
                &offer.description,
                &offer.expiration_time,
            ]
        ).await {
            Ok(x) => x,
            Err(db_error) => { return Err(gen_dao_error("save_offer", db_error)); }
        };
        let offer_id =  row.get("offerId");
        offer.offer_id = offer_id;
        Ok(())
    }
}
