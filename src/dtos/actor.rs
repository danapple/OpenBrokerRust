use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use strum_macros::EnumIter;

#[derive(Debug, Deserialize, Serialize, Clone, ToSql, FromSql, PartialEq, EnumIter)]
pub enum Power {
    All,
    Read,
}

impl Display for Power {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Power {
    type Err = ();
    fn from_str(input: &str) -> Result<Power, Self::Err> {
        match input {
            "All"  => Ok(Power::All),
            "Read"  => Ok(Power::Read),
            _  => Err(()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Actor {
    pub email: String,
}
