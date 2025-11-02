use crate::dtos::actor::Power;
use crate::persistence::dao::{gen_dao_error, DaoError, DaoTransaction};
use std::str::FromStr;

impl<'b> DaoTransaction<'b> {

    pub async fn get_powers(&self, actor_id: i32) -> Result<Vec<Power>, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(POWER_QUERY);
        query_string.push_str(" WHERE admin_role_membership.actorId = $1");
        let rows = match self.transaction.query(&query_string,
                                                &[&actor_id]).await {
            Ok(x) => x,
            Err(db_error) => { return Err(gen_dao_error("get_accounts", db_error)); }
        };
        let mut powers: Vec<Power> = Vec::new();
        for row in rows {
            let power_string = row.get("power");
            let power = match Power::from_str(power_string){
                Ok(power) => power,
                Err(()) => {
                    return Err(DaoError::ConversionFailed {
                        description: format!("Unknown power {}", power_string)
                    })
                }
            };

            powers.push(power);
        }
        Ok(powers)
    }
}

const POWER_QUERY: &str = "\
SELECT power \
FROM admin_role_power \
JOIN admin_role_membership ON admin_role_membership.adminRoleId = admin_role_power.adminRoleId \
";
