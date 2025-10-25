use crate::entities::account::Position;
use crate::persistence::dao::{DaoError, DaoTransaction};
use log::error;
use std::collections::HashMap;
use tokio_postgres::Row;

impl<'b> DaoTransaction<'b> {
    pub async fn get_positions(&self, account_key: &String) -> Result<HashMap<i64, Position>, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(POSITION_QUERY);
        query_string.push_str("WHERE account.accountKey = $1");
        let res = match self.transaction.query(&query_string,
                                               &[&account_key]).await {
            Ok(x) => x,
            Err(y) => { error!("get_positions {}: {}", y.to_string(), match y.as_db_error() {Some(x) => format!("{}", x),None => "none".parse().unwrap()}); return Err(DaoError::ExecuteFailed { description: y.to_string() })},
        };

        let positions_map = convert_rows_to_positions(res);
        Ok(positions_map)
    }

    pub async fn get_position(&self, account_key: &String, instrument_id: i64) -> Result<Option<Position>, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(POSITION_QUERY);
        query_string.push_str("WHERE account.accountKey = $1 AND position.instrumentId = $2");
        let res = match self.transaction.query(&query_string,
                                               &[
                                                   &account_key,
                                                   &instrument_id]).await {
            Ok(x) => x,
            Err(y) => { error!("get_position {}: {}", y.to_string(), match y.as_db_error() {Some(x) => format!("{}", x),None => "none".parse().unwrap()}); return Err(DaoError::ExecuteFailed { description: y.to_string() })},
        };

        let positions_map = convert_rows_to_positions(res);
        let position = match positions_map.get(&instrument_id) {
            None => { None }
            Some(pos) => { Some(pos.clone()) }
        };
        Ok(position)
    }

    pub async fn update_position(&self, position: &mut Position) -> Result<(), DaoError> {
        let next_version_number = position.version_number + 1;
        let rows_updated = match self.transaction.execute(POSITION_UPDATE_STATEMENT,
                                                          &[
                                                              &position.cost,
                                                              &position.quantity,
                                                              &position.closed_gain,
                                                              &position.update_time,
                                                              &next_version_number,
                                                              &position.position_id,
                                                              &position.version_number
                                                          ]
        ).await {
            Ok(rows) => rows,
            Err(y) => { error!("update_position {}: {}", y.to_string(), match y.as_db_error() {Some(x) => format!("{}", x),None => "none".parse().unwrap()}); return Err(DaoError::ExecuteFailed { description: y.to_string() })},
        };
        if rows_updated == 0 {
            return Err(DaoError::OptimisticLockingFailed{ description: "update position 0 rows modified".to_string() });
        }
        position.version_number = next_version_number;
        Ok(())
    }

    pub async fn save_position(&self, mut position: Position) -> Result<(Position), DaoError> {
        let row = match self.transaction.query_one(
            POSITION_SAVE_STATEMENT,
            &[
                &position.account_id,
                &position.instrument_id,
                &position.cost,
                &position.quantity,
                &position.closed_gain,
                &position.update_time,
                &position.version_number
            ]).await {
            Ok(x) => x,
            Err(y) => { error!("save_position {}: {}", y.to_string(), match y.as_db_error() {Some(x) => format!("{}", x),None => "none".parse().unwrap()}); return Err(DaoError::ExecuteFailed { description: y.to_string() })},
        };
        let position_id =  row.get("positionId");
        position.position_id = position_id;
        Ok(position)
    }
}

fn convert_rows_to_positions(rows: Vec<Row>) -> HashMap<i64, Position> {
    let mut positions = HashMap::new();
    for row in rows {
        let position = convert_row_to_position(row);
        positions.insert(position.instrument_id, position);
    }
    positions
}


fn convert_row_to_position(row: Row) -> Position {
    Position {
        position_id: row.get("positionId"),
        account_id: row.get("accountId"),
        instrument_id: row.get("instrumentId"),
        quantity: row.get("quantity"),
        cost: row.get("cost"),
        closed_gain: row.get("closedGain"),
        update_time: row.get("updateTime"),
        version_number: row.get("versionNumber"),
    }
}

const POSITION_SAVE_STATEMENT: &str = "
INSERT INTO position \
(accountId, instrumentId, cost, quantity, closedGain, updateTime, versionNumber) \
VALUES \
($1, $2, $3, $4, $5, $6, $7) \
RETURNING positionId
";

const POSITION_QUERY: &str = "
SELECT positionId, position.accountId, instrumentId, cost, quantity, closedGain, updateTime, versionNumber FROM position \
JOIN account on account.accountId = position.accountId \
";

const POSITION_UPDATE_STATEMENT: &str = "
UPDATE position
SET cost = $1, quantity = $2, closedGain = $3, updateTime = $4, versionNumber = $5 \
WHERE position.positionId = $6 AND position.versionNumber = $7 \
";

