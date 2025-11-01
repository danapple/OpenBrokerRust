use crate::entities;
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use strum_macros::EnumIter;

#[derive(Debug, Deserialize, Serialize, Clone, ToSql, FromSql, PartialEq, EnumIter)]
pub enum InstrumentStatus {
    Active,
    Inactive,
}

impl Display for InstrumentStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for InstrumentStatus {
    type Err = ();
    fn from_str(input: &str) -> Result<InstrumentStatus, Self::Err> {
        match input {
            "Active"  => Ok(InstrumentStatus::Active),
            "Inactive"  => Ok(InstrumentStatus::Inactive),
            _  => Err(()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSql, FromSql, PartialEq, EnumIter)]
pub enum AssetClass {
    Equity,
    Option,
    Commodity,
    Future,
    Forward,
    Swap,
    Bond,
    Cryto,
}

impl Display for AssetClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for AssetClass {
    type Err = ();
    fn from_str(input: &str) -> Result<AssetClass, Self::Err> {
        match input {
            "Equity"  => Ok(AssetClass::Equity),
            "Option"  => Ok(AssetClass::Option),
            "Commodity"  => Ok(AssetClass::Commodity),
            "Future"  => Ok(AssetClass::Future),
            "Forward"  => Ok(AssetClass::Forward),
            "Swap"  => Ok(AssetClass::Swap),
            "Bond"  => Ok(AssetClass::Bond),
            "Cryto"  => Ok(AssetClass::Cryto),
            _  => Err(()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSql, FromSql, PartialEq)]
pub struct Instrument {
    pub instrument_key: String,
    pub status: InstrumentStatus,
    pub symbol: String,
    pub asset_class: AssetClass,
    pub exchange_code: String,
    pub description: String,
    pub expiration_time: i64
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Exchange {
    pub code: String,
    pub url: String,
    pub websocket_url: String,
    pub description: String,
    pub api_key: String,

}

impl Exchange {
    pub fn to_entities_exchange(&self) -> entities::exchange::Exchange {
        entities::exchange::Exchange {
            exchange_id: 0,
            code: self.code.clone(),
            url: self.url.clone(),
            websocket_url: self.websocket_url.clone(),
            description: self.description.clone(),
            api_key: self.api_key.clone(),
        }
    }
}