use crate::config::BrokerConfig;
use crate::exchange_interface::exchange_error::ExchangeError;
use crate::exchange_interface::trading::{Instruments, Order, OrderState, OrderStates, SubmitOrders};
use log::debug;
use reqwest::cookie::Jar;
use reqwest::{Client, Response};
use std::sync::Arc;
use url::Url;

pub struct ExchangeClient {
    client: Client,
    exchange_url: String,
}

pub(crate) fn get_customer_key_cookie(customer_key: &String) -> String {
    ["customerKey", &customer_key].join("=")
}

impl ExchangeClient {
    pub fn new (config: &BrokerConfig) -> Self {
        let jar = Arc::new(Jar::default());
        let url = match config.exchange_url.parse::<Url>() {
            Ok(url_string) => url_string,
            Err(parse_error) => {
                panic!("ExchangeClient::new url_string parse error {}", parse_error);
            }
        };
        jar.add_cookie_str(&get_customer_key_cookie(&config.broker_key), &url);

        let client = match Client::builder().cookie_provider(Arc::clone(&jar)).build() {
            Ok(client) => client,
            Err(reqwest_error) => {
                panic!("ExchangeClient::new Client::builder().build error {}", reqwest_error);
            },
        };

        ExchangeClient {
            client,
            exchange_url: config.exchange_url.clone(),
        }
    }

    fn get_url(&self, region: &str) -> Result<Url, ExchangeError> {
        let base_url = &self.exchange_url;
        let full_url = format!("{base_url}/{region}");
        match full_url.parse::<Url>() {
            Ok(url) => Ok(url),
            Err(parse_error) =>
                Err(ExchangeError::Failure { description: "parsing url".to_string(), cause: parse_error.to_string() }),
        }
    }

    fn get_url_with_id(&self, region: &str, client_order_id: &String) -> Result<Url, ExchangeError> {
        let base_url = &self.exchange_url;
        let full_url = format!("{base_url}/{region}/{client_order_id}");
        match full_url.parse::<Url>() {
            Ok(url) => Ok(url),
            Err(parse_error) =>
                Err(ExchangeError::Failure { description: "parsing url with id".to_string(), cause: parse_error.to_string() }),        }
    }

    pub async fn get_instruments(&self) -> Result<Instruments, ExchangeError> {
        let instruments_url = match self.get_url("instruments") {
            Ok(instruments_url) => instruments_url,
            Err(url_error) => return Err(url_error),
        };
        let send = self.client.get(instruments_url).send();

        debug!("About to send for instruments");
        let response = match send.await {
            Ok(response) => {
                response
            },
            Err(parse_error) =>
                return Err(ExchangeError::Failure { description: "send".to_string(), cause: parse_error.to_string() }),
        };
        debug!("Got instruments");

        match response.json::<Instruments>().await {
            Ok(instruments) => Ok(instruments),
            Err(json_error) => Err(ExchangeError::Failure { description: "json".to_string(), cause: json_error.to_string() }),
        }
    }

    pub async fn submit_order(&self, order: Order) -> Result<OrderState, ExchangeError> {
        let orders = SubmitOrders { orders: vec![order] };
        let url = match self.get_url("orders") {
            Ok(url) => url,
            Err(get_url_error) => return Err(ExchangeError::Failure { description: "submit_order get_url".to_string(), cause: get_url_error.to_string() })
        };
        let send = self.client.post(url).json(&orders).send();

        Self::execute(send).await
    }


    pub async fn cancel_order(&self, client_order_id: String) -> Result<OrderState, ExchangeError> {
        let url = match self.get_url_with_id("orders", &client_order_id) {
            Ok(url) => url,
            Err(get_url_error) => return Err(ExchangeError::Failure { description: "cancel_order get_url_with_id".to_string(), cause: get_url_error.to_string() })
        };
        let send = self.client.delete(url).send();

        Self::execute(send).await
    }

    async fn execute(send: impl Future<Output=Result<Response, reqwest::Error>>) -> Result<OrderState, ExchangeError> {
        let response = match send.await {
            Ok(response) => response,
            Err(send_error) => return Err(ExchangeError::Failure { description: "send await".to_string(), cause: send_error.to_string() })
        };

        let order_states = match response.json::<OrderStates>().await {
            Ok(order_states) => order_states,
            Err(send_error) => return Err(ExchangeError::Failure { description: "json".to_string(), cause: send_error.to_string() })
        };

        if order_states.order_states.len() != 1 {
            return Err(ExchangeError::Failure { description: "Incorrect number of order states returned".to_string(), cause: format!("{} instead of 1", order_states.order_states.len()) })
        }

        match order_states.order_states.first() {
            Some(order_state) => Ok(order_state.clone()),
            None => Err(ExchangeError::Failure { description: "first".to_string(), cause: "No order_state available".to_string() })
        }
    }
}
