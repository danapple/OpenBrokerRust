use crate::entities::exchange::{Exchange, Instrument};
use crate::exchange_interface::exchange_client::ExchangeClient;
use crate::exchange_interface::websocket_client::ExchangeWebsocketClient;
use crate::market_data::receiver::{handle_depth, handle_last_trade};
use crate::persistence::dao::{Dao, DaoTransaction};
use crate::trade_handling::execution_handling::handle_execution;
use crate::trade_handling::order_state_handling::handle_order_state;
use crate::websockets::server::WebSocketServer;
use anyhow::Error;
use log::info;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct InstrumentManager {
    dao: Dao,
    web_socket_server: WebSocketServer,
    instruments: Arc<RwLock<HashMap<i64, Instrument>>>,
    instruments_by_exchange_instrument_id: Arc<RwLock<HashMap<i64, Instrument>>>,
    exchanges_holders_by_id: Arc<RwLock<HashMap<i32, Arc<ExchangeHolder>>>>,
}

struct ExchangeHolder {
    exchange_client: Arc<ExchangeClient>,
    exchange_websocket_client: Arc<ExchangeWebsocketClient>
}

impl InstrumentManager {
    pub fn new (dao: Dao, web_socket_server: WebSocketServer) -> Self {
        InstrumentManager {
            dao,
            web_socket_server,
            instruments: Arc::new(RwLock::new(HashMap::new())),
            instruments_by_exchange_instrument_id: Arc::new(RwLock::new(HashMap::new())),
            exchanges_holders_by_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub async fn initialize(&mut self) -> Result<(), Error> {
        let mut db_connection = match self.dao.get_connection().await {
            Ok(x) => x,
            Err(dao_error) => panic!("Could not get_connection: {}", dao_error),
        };
        let txn = match self.dao.begin(&mut db_connection).await {
            Ok(x) => x,
            Err(dao_error) => panic!("Could not begin: {}", dao_error),
        };

        match self.load_exchanges(&txn).await {
            Ok(_) => { },
            Err(err) => panic!("Could not load exchanges: {}", err),
        };

        match self.load_instruments(&txn).await {
            Ok(x) => x,
            Err(err) => panic!("Could not load instruments: {}", err),
        };

        match txn.rollback().await {
            Ok(_) => Ok(()),
            Err(dao_error) => panic!("Could not rollback: {}", dao_error),
        }
    }

    async fn load_exchanges(&self, txn: &DaoTransaction<'_>)  -> Result<(), Error> {
        let exchanges = match txn.get_exchanges().await {
            Ok(exchanges) => exchanges,
            Err(dao_error) => return Err(anyhow::anyhow!("Could not get_exchanges: {}", dao_error)),
        };
        for exchange in exchanges.values() {
            info!("Adding exchange {}", exchange.code);
            match self.setup_exchange(exchange.clone()).await {
                Ok(x) => x,
                Err(setup_error) => return Err(anyhow::anyhow!("Could not set up exchange {}: {}", exchange.code, setup_error)),
            };
        }
        info!("Done adding exchanges");
        Ok(())
    }

    async fn load_instruments(&mut self, txn: &DaoTransaction<'_>) -> Result<(), Error> {
        let instruments = match txn.get_instruments().await {
            Ok(x) => x,
            Err(dao_error) => return Err(anyhow::anyhow!("Could not get instruments: {}", dao_error)),
        };

        for instrument in instruments.values() {
            info!("Adding instrument: {} for exchange {}", instrument.instrument_id, instrument.exchange_id);
            match self.add_instrument(instrument) {
                Ok(_) => {}
                Err(err) => return Err(err)
            }
        }

        info!("Done adding instruments");
        Ok(())
    }

    pub async fn setup_exchange(&self, exchange: Exchange) -> Result<(), Error> {
        let exchange_client = ExchangeClient::new(exchange.url.as_str(), exchange.api_key.as_str());
        let exchange_websocket_client = ExchangeWebsocketClient::new(exchange.websocket_url.clone(),
                                                                     exchange.api_key.clone(),
                                                                     self.dao.clone(),
                                                                     self.web_socket_server.clone(),
                                                                     self.clone(),
                                                                     handle_execution, handle_order_state,
                                                                     handle_depth, handle_last_trade);
        exchange_websocket_client.start_exchange_websockets().await;

        let exchange_holder = ExchangeHolder {
            exchange_client: Arc::new(exchange_client),
            exchange_websocket_client: Arc::new(exchange_websocket_client),
        };
        let mut writable_exchanges = match self.exchanges_holders_by_id.write() {
            Ok(writable_exchanges) => writable_exchanges,
            Err(writable_error) => return Err(anyhow::anyhow!("Unable to get write access to exchanges: {}", writable_error)),
        };
        writable_exchanges.insert(exchange.exchange_id, Arc::from(exchange_holder));

        Ok(())
    }

    pub fn get_exchange_client_for_instrument(&self, instrument: &Instrument) -> Result<Arc<ExchangeClient>, Error> {
        let readable_exchanges = match self.exchanges_holders_by_id.read() {
            Ok(readable_exchanges) => readable_exchanges,
            Err(readable_error) => return Err(anyhow::anyhow!("Unable to get read access to exchanges: {}", readable_error)),
        };
        match readable_exchanges.get(&instrument.exchange_id) {
            Some(exchange_holder) => Ok(exchange_holder.exchange_client.clone()),
            None => Err(anyhow::anyhow!("No exchange for instrument: {}", instrument.instrument_id)),
        }
    }

    pub fn add_instrument(&mut self, instrument: &Instrument) -> Result<(), Error> {
        let mut writable_instruments = match self.instruments.write() {
            Ok(writable_instruments) => writable_instruments,
            Err(writable_error) => return Err(anyhow::anyhow!("Unable to get write access to instruments: {}", writable_error)),
        };
        writable_instruments.insert(instrument.instrument_id, instrument.clone());

        let mut writable_instruments_by_exchange_instrument_id = match self.instruments_by_exchange_instrument_id.write() {
            Ok(writable_instruments_by_exchange_instrument_id) => writable_instruments_by_exchange_instrument_id,
            Err(writable_error) => return Err(anyhow::anyhow!("Unable to get write access to instruments_by_exchange_instrument_id: {}", writable_error)),
        };
        writable_instruments_by_exchange_instrument_id.insert(instrument.exchange_instrument_id, instrument.clone());
        Ok(())
    }

    pub fn get_instrument(&self, instrument_id: i64) -> Result<Option<Instrument>, Error> {
        let instruments = match self.instruments.read() {
            Ok(x) => x,
            Err(writable_error) => return Err(anyhow::anyhow!("get_instrument unable to get read access to instruments: {}", writable_error)),
        };
        match instruments.get(&instrument_id) {
            Some(instrument) => Ok(Some(instrument.clone())),
            None => Ok(None)
        }
    }

    pub fn get_instrument_by_exchange_instrument_id(&self, exchange_instrument_id: i64) -> Result<Option<Instrument>, anyhow::Error> {
        let instruments = match self.instruments_by_exchange_instrument_id.read() {
            Ok(x) => x,
            Err(writable_error) => return Err(anyhow::anyhow!("get_instrument_by_exchange_instrument_id unable to get read access to instruments: {}", writable_error)),
        };
        match instruments.get(&exchange_instrument_id) {
            Some(instrument) => Ok(Some(instrument.clone())),
            None => Ok(None)
        }
    }
}