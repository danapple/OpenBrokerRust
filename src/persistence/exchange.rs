use crate::entities::exchange::{Exchange, Instrument};
use crate::persistence::dao::{gen_dao_error, DaoError, DaoTransaction};
use crate::rest_api::exchange::{AssetClass, InstrumentStatus};
use std::collections::HashMap;
use std::str::FromStr;
use tokio_postgres::Row;


impl<'b> DaoTransaction<'b> {
    pub async fn save_exchange(&self, exchange: &mut Exchange) -> Result<(), DaoError> {
        let row = match self.transaction.query_one(
            "INSERT INTO exchange \
            (code, url, websocketUrl, description, apiKey) \
            VALUES ($1, $2, $3, $4, $5) \
            RETURNING exchangeId",
            &[&exchange.code,
                &exchange.url,
                &exchange.websocket_url,
                &exchange.description,
                &exchange.api_key
            ]
        ).await {
            Ok(x) => x,
            Err(db_error) => { return Err(gen_dao_error("save_exchange exchange", db_error)); }
        };
        let exchange_id: i32 = row.get("exchangeId");

        exchange.exchange_id = exchange_id;

        Ok(())
    }

    pub async fn get_exchange(&self, exchange_code: &str) -> Result<Exchange, DaoError> {
        let row = match self.transaction.query_one(
            "SELECT exchangeId, code, url, websocketUrl, description, apiKey FROM exchange \
            WHERE code = $1",
            &[&exchange_code
            ]
        ).await {
            Ok(x) => x,
            Err(db_error) => { return Err(gen_dao_error("get_exchange exchange", db_error)); }
        };

        Ok(Exchange {
            exchange_id: row.get("exchangeId"),
            code: row.get("code"),
            url: row.get("url"),
            websocket_url: row.get("websocketUrl"),
            description: row.get("description"),
            api_key: row.get("apiKey"),
        })
    }

    pub async fn get_exchanges(&self) -> Result<HashMap<i32, Exchange>, DaoError> {
        let rows = match self.transaction.query(EXCHANGE_QUERY,
                                                &[]).await {
            Ok(rows) => rows,
            Err(db_error) => { return Err(gen_dao_error("get_exchanges", db_error)); }

        };

        let exchanges = match convert_rows_to_exchanges(rows) {
            Ok(exchanges) => exchanges,
            Err(db_error) => { return Err(db_error) }
        };
        Ok(exchanges)
    }

    pub async fn save_instrument(&self, instrument: &mut Instrument) -> Result<(), DaoError> {
        let row = match self.transaction.query_one(
            "INSERT INTO instrument \
            (instrumentKey, exchangeId, exchangeInstrumentId, status, symbol, assetClass, description, expirationTime) \
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
            ON CONFLICT (exchangeId, exchangeInstrumentId)
            DO UPDATE \
            SET status = $4,\
             symbol = $5,\
             assetClass = $6,\
             description = $7,\
             expirationTime = $8 \
            RETURNING instrumentId",
            &[&instrument.instrument_key,
                &instrument.exchange_id,
                &instrument.exchange_instrument_id,
                &instrument.status.to_string(),
                &instrument.symbol,
                &instrument.asset_class.to_string(),
                &instrument.description,
                &instrument.expiration_time
            ]
        ).await {
            Ok(x) => x,
            Err(db_error) => { return Err(gen_dao_error("save_instrument instrument", db_error)); }
        };
        let instrument_id = row.get("instrumentId");
        instrument.instrument_id = instrument_id;

        Ok(())
    }

    pub async fn get_instruments(&self) -> Result<HashMap<i64, Instrument>, DaoError> {
        let res = match self.transaction.query(INSTRUMENT_QUERY,
                                               &[]).await {
            Ok(x) => x,
            Err(db_error) => { return Err(gen_dao_error("get_instruments", db_error)); }

        };

        let instruments = match convert_rows_to_instruments(res) {
            Ok(instruments) => instruments,
            Err(db_error) => { return Err(db_error) }
        };
        Ok(instruments)
    }
}

fn convert_rows_to_exchanges(rows: Vec<Row>) -> Result<HashMap<i32, Exchange>, DaoError> {
    let mut exchanges = HashMap::new();
    for row in rows {
        let exchange = convert_row_to_exchange(&row);
        exchanges.insert(exchange.exchange_id, exchange.clone());
    }

    Ok(exchanges)
}

fn convert_row_to_exchange(row: &Row) -> Exchange {
    Exchange {
        exchange_id: row.get("exchangeId"),
        code: row.get("code"),
        url: row.get("url"),
        websocket_url: row.get("websocketUrl"),
        description: row.get("description"),
        api_key: row.get("apiKey"),
    }
}

fn convert_rows_to_instruments(rows: Vec<Row>) -> Result<HashMap<i64, Instrument>, DaoError> {
    let mut instruments = HashMap::new();
    for row in rows {
        let instrument = match convert_row_to_instrument(&row) {
            Ok(instrument) => instrument,
            Err(db_error) => { return Err(db_error) }
        };

        instruments.insert(instrument.instrument_id, instrument.clone());
    }

    Ok(instruments)
}

fn convert_row_to_instrument(row: &Row) -> Result<Instrument, DaoError> {
    let row_instrument_status = row.get("status");
    let instrument_status = match InstrumentStatus::from_str(row_instrument_status) {
        Ok(instrument_status) => instrument_status,
        Err(err) =>  return Err(DaoError::ConversionFailed { description: format!("Could not parse instrument status {}", row_instrument_status) })
    };
    let row_asset_class = row.get("assetClass");
    let asset_class = match AssetClass::from_str(row_asset_class) {
        Ok(asset_class) => asset_class,
        Err(err) =>  return Err(DaoError::ConversionFailed { description: format!("Could not parse asset class {}", row_asset_class) })
    };

    Ok(Instrument {
        instrument_id: row.get("instrumentId"),
        instrument_key: row.get("instrumentKey"),
        exchange_id: row.get("exchangeId"),
        exchange_instrument_id: row.get("exchangeInstrumentId"),
        status: instrument_status,
        symbol: row.get("symbol"),
        asset_class,
        description: row.get("description"),
        expiration_time: row.get("expirationTime"),
    })
}

const EXCHANGE_QUERY: &str = "SELECT exchangeId, code, url, \
websocketUrl, description, apiKey \
FROM exchange \
";

const INSTRUMENT_QUERY: &str = "SELECT instrumentId, instrumentKey, exchangeId, exchangeInstrumentId, \
status, symbol, assetClass, description, expirationTime \
FROM instrument \
";
