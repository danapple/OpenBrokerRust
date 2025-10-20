use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use anyhow::Error;

use crate::config::BrokerConfig;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use anyhow::Result;
use futures_util::SinkExt;
use log::{error, info};
use tokio_stream::StreamExt;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use crate::websockets::stomp;
use crate::websockets::stomp::parse_message;
pub(crate) use crate::websockets::stomp::StompMessage;

pub struct WebsocketClient {
    websocket_address: String,
    broker_key: String,
    handlers: Arc<RwLock<HashMap<String, Arc<dyn Fn(&StompMessage) + Send + Sync + 'static>>>>,
}

impl WebsocketClient {
    pub fn new<'e>(config: &BrokerConfig) -> Self {
        WebsocketClient {
            websocket_address: config.exchange_websocket_address.clone(),
            broker_key: config.broker_key.clone(),
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn subscribe(&mut self, destination: &str, func: Arc<dyn Fn(&StompMessage) + Send + Sync + 'static>){
        self.handlers.write().unwrap().insert(destination.to_string(), func);
    }

    pub fn start(&mut self) {
        let f = listen(self.websocket_address.clone(), self.broker_key.clone(), self.handlers.clone());
        tokio::spawn(f);
    }
}

async fn listen(websocket_address: String, broker_key: String, handlers: Arc<RwLock<HashMap<String, Arc<dyn Fn(&StompMessage) + Send + Sync + 'static>>>>) -> Result<()> {
    let mut unboxed_handlers = HashMap::new();
    for (key, value) in handlers.read().unwrap().iter() {
        unboxed_handlers.insert(key.clone(), value.to_owned());
    }

    let mut request = match websocket_address.into_client_request() {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    request.headers_mut().insert("Cookie",
                                 match format!("customerKey={}", broker_key).parse() {
                                     Ok(x) => x,
                                     Err(_) => todo!(),
                                 });

    let (mut ws_stream, _) = connect_async(request).await.expect("Failed to connect");
    println!("WebSocket client connected");
    ws_stream.send(stomp::connect()).await?;

    for (destination, func) in unboxed_handlers.iter() {
        ws_stream.send(stomp::subscribe_message(destination)).await?;
    }

    while let Some(msg) = ws_stream.next().await {
        match msg? {
            Message::Text(text) => {
                if text.starts_with("MESSAGE") {
                    let message = parse_message(&text);
                    match unboxed_handlers.get(message.destination.as_str()) {
                        Some(func) => func(&message),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

