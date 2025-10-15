use crate::entities::account::{Balance, Position};
use crate::persistence::dao::{DaoError, DaoTransaction};

impl<'b> DaoTransaction<'b> {
    pub async fn get_positions(&self, account_key: &String) -> Result<Vec<Position>, DaoError> {

        todo!()
    }
}


impl<'b> DaoTransaction<'b> {
    pub async fn get_balances(&self, account_key: &String) -> Result<Balance, DaoError> {
        todo!()
    }
}