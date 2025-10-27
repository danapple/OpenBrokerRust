use crate::exchange_interface::exchange_client::ExchangeClient;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct InstrumentManager {
    instruments: Arc<RwLock<HashMap<i64, Instrument>>>,
    instruments_by_exchange_instrument_id: Arc<RwLock<HashMap<i64, Instrument>>>,
    next_instrument_id: Arc<RwLock<i64>>,
}

#[derive(Clone)]
pub struct Instrument {
    pub instrument_id: i64,
    pub exchange_instrument_id: i64,
    pub exchange_client: Arc<ExchangeClient>,
    // pub value_factor: f64,
    // pub underlying_instrument_id: u64,
    // pub underlying_quantity: f64,
}

impl InstrumentManager {
    pub fn new () -> Self {
        InstrumentManager {
            instruments: Arc::new(RwLock::new(HashMap::new())),
            instruments_by_exchange_instrument_id: Arc::new(RwLock::new(HashMap::new())),
            next_instrument_id: Arc::new(RwLock::new(0)),
        }
    }

    pub fn add_instrument(&mut self, exchange_instrument_id: i64, exchange_client: Arc<ExchangeClient>) -> Result<i64, anyhow::Error> {
        let mut next_instrument_id = match self.next_instrument_id.write() {
            Ok(x) => x,
            Err(writable_error) => return Err(anyhow::anyhow!("Unable to get write access to next_instrument_id: {}", writable_error)),
        };
        let instrument_id = next_instrument_id.clone();
        *next_instrument_id += 1;

        let instrument = Instrument{
            instrument_id,
            exchange_instrument_id,
            exchange_client,
        };
        match self.instruments.write() {
            Ok(x) => x,
            Err(writable_error) => return Err(anyhow::anyhow!("Unable to get write access to instruments: {}", writable_error)),
        }.insert(instrument_id.clone(), instrument.clone());
        match self.instruments_by_exchange_instrument_id.write() {
            Ok(x) => x,
            Err(writable_error) => return Err(anyhow::anyhow!("Unable to get write access to instruments_by_exchange_instrument_id: {}", writable_error)),
        }.insert(exchange_instrument_id, instrument);
        Ok(instrument_id.clone())
    }

    pub fn get_instrument(&self, instrument_id: i64) -> Result<Option<Instrument>, anyhow::Error> {
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