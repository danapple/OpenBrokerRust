use std::sync::Arc;
use reqwest::{Client, Response};
use reqwest::cookie::Jar;
use url::{Url};
use crate::exchange_interface::trading::{Order, OrderState, SubmitOrders, OrderStates, Instruments};
use log::{error, info};
use crate::config::BrokerConfig;

pub struct ExchangeClient {
    client: Client,
    exchange_url: String,
}

fn get_customer_key_cookie(broker_key: &String) -> String {
    let customer_key = broker_key.clone();
    ["customerKey", &customer_key].join("=")
}

impl ExchangeClient {
    pub fn new (config: &BrokerConfig) -> Self {
        let jar = Arc::new(Jar::default());
        let url_string = match config.exchange_url.parse::<Url>() {
            Ok(x) => {x}
            Err(_) => todo!()
        };
        jar.add_cookie_str(&get_customer_key_cookie(&config.broker_key), &url_string);

        let client = match Client::builder().cookie_provider(Arc::clone(&jar)).build() {
            Ok(x) => x,
            Err(_) => todo!(),
        };

        ExchangeClient {
            client,
            exchange_url: config.exchange_url.clone(),
        }
    }


    fn get_url(&self, region: &str) -> Url {
        let base_url = &self.exchange_url;
        let full_url = format!("{base_url}/{region}");
        let url = match full_url.parse::<Url>() {
            Ok(x) => x,
            Err(_) => todo!(),
        };

        url
    }

    fn get_url_with_id(&self, region: &str, client_order_id: &String) -> Url {
        let base_url = &self.exchange_url;
        let full_url = format!("{base_url}/{region}/{client_order_id}");
        let url = match full_url.parse::<Url>() {
            Ok(x) => x,
            Err(_) => todo!(),
        };

        url
    }

    pub async fn get_instruments(&self) -> Instruments {
        let send = self.client.get(self.get_url("instruments")).send();

        info!("About to send");
        let response = match send.await {
            Ok(x) => {info!("Ok"); x},
            Err(_) => {error!("Fail");todo!()},
        };
        info!("Got instrument");

        response.json::<Instruments>().await.unwrap_or_else(|y| {
            error!("err {}", y);
            todo!();
        })
    }

    pub async fn submit_order(&self, order: Order) -> OrderState {
        let orders = SubmitOrders { orders: vec![order] };
        let send = self.client.post(self.get_url("orders")).json(&orders).send();
        Self::execute(send).await
    }


    pub async fn cancel_order(&self, client_order_id: String) -> OrderState {
        let send = self.client.delete(self.get_url_with_id("orders", &client_order_id)).send();
        Self::execute(send).await
    }

    async fn execute(send: impl Future<Output=Result<Response, reqwest::Error>>) -> OrderState {
        let response = match send.await {
            Ok(x) => x,
            Err(_) => todo!(),
        };

        let order_states = response.json::<OrderStates>().await.unwrap_or_else(|y| {
            error!("err {}", y);
            todo!();
        });

        match order_states.order_states.first() {
            Some(x) => x.clone(),
            None => todo!(),
        }
    }
}
