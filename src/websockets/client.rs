use anyhow::Error;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::config::BrokerConfig;
use crate::websockets::stomp;
pub(crate) use crate::websockets::stomp::StompMessage;
use crate::websockets::stomp::{parse_message, MessageContent};
use anyhow::Result;
use futures_util::SinkExt;
use log::{debug, error, info};
use tokio_stream::StreamExt;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

pub struct WebsocketClient {
    websocket_address: String,
    broker_key: String,
    handlers: Arc<RwLock<HashMap<String, Arc<dyn Fn(&MessageContent) + Send + Sync + 'static>>>>,
}

impl WebsocketClient {
    pub fn new<'e>(config: &BrokerConfig) -> Self {
        WebsocketClient {
            websocket_address: config.exchange_websocket_address.clone(),
            broker_key: config.broker_key.clone(),
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn subscribe(&mut self, destination: &str, func: Arc<dyn Fn(&MessageContent) + Send + Sync + 'static>){
        self.handlers.write().unwrap().insert(destination.to_string(), func);
    }

    pub fn start(&mut self) {
        let f = listen(self.websocket_address.clone(), self.broker_key.clone(), self.handlers.clone());
        tokio::spawn(f);
    }
}

async fn listen(websocket_address: String, broker_key: String, handlers: Arc<RwLock<HashMap<String, Arc<dyn Fn(&MessageContent) + Send + Sync + 'static>>>>) -> Result<()> {
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
    ws_stream.send(stomp::connect_message()).await?;

    let mut subscription_id = 0;
    while let Some(msg) = ws_stream.next().await {
        match msg? {
            Message::Text(text) => {
                debug!("Received message: {} on client", text);
                match parse_message(&text) {
                    StompMessage::Message(msg) => {
                        debug!("Handling StompMessage:Message");
                        match unboxed_handlers.get(msg.destination.as_str()) {
                            Some(func) => func(&msg),
                            _ => {}
                        }
                    }
                    StompMessage::Connected(ct) => {
                        debug!("Received expected Connected message on client");
                        for (destination, func) in unboxed_handlers.iter() {
                            debug!("Subscribing to {}", destination);
                            ws_stream.send(stomp::subscribe_message(subscription_id, destination)).await?;
                            subscription_id += 1
                        }
                    }
                    StompMessage::Subscribe(sub) => {
                        error!("Received unexpected subscribe message on client: {}", sub.destination);
                    },
                    StompMessage::Connect(ct) => {
                        error!("Received unexpected Connect message on client: {}", ct.accept_version);
                    },
                };
            }
            y => {
                info!("Received unexpected non-text message on client: {:?}", y);
            }
        }
    }
    Ok(())
}

