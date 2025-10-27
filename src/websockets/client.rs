use log::trace;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};

use crate::config::BrokerConfig;
use crate::exchange_interface::exchange_client;
use crate::exchange_interface::exchange_client::get_customer_key_cookie;
use crate::websockets::stomp;
pub(crate) use crate::websockets::stomp::StompMessage;
use crate::websockets::stomp::{parse_message, MessageContent};
use anyhow::Result;
use futures_util::SinkExt;
use log::{debug, error, info};
use tokio_stream::StreamExt;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

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
        let mut writable_handlers = match self.handlers.write() {
            Ok(writable_handlers) => writable_handlers,
            Err(poison_error) => {
                error!("Could not get writable handlers to subscribe to {}: {}", destination, poison_error.to_string());
                return;
            },
        };
        writable_handlers.insert(destination.to_string(), func);
    }

    pub fn start(&mut self) {
        let f = listen(self.websocket_address.clone(), self.broker_key.clone(), self.handlers.clone());
        tokio::spawn(f);
    }
}

async fn listen(websocket_address: String, broker_key: String, handlers: Arc<RwLock<HashMap<String, Arc<dyn Fn(&MessageContent) + Send + Sync + 'static>>>>) -> Result<()> {
    let mut unboxed_handlers = HashMap::new();
    match unbox_handlers(handlers, &mut unboxed_handlers) {
        Ok(_) => { },
        Err(poison_error) => {
            return Err(anyhow::anyhow!("Could not to get readable handlers: {}", poison_error.to_string()));
        },
    };

    let mut request = match websocket_address.into_client_request() {
        Ok(request) => request,
        Err(request_error) =>
            return Err(anyhow::anyhow!("Could not create client request: {}", request_error.to_string())),
    };

    let customer_key_cookie = match get_customer_key_cookie(&broker_key).parse() {
        Ok(customer_key_cookie) => customer_key_cookie,
        Err(parse_error) =>
            return Err(anyhow::anyhow!("Could parse customer key cookie: {}", parse_error)),
    };
    request.headers_mut().insert("Cookie", customer_key_cookie);

    let (mut ws_stream, _) = connect_async(request).await.expect("Failed to connect");
    println!("WebSocket client connected");
    ws_stream.send(stomp::connect_message()).await?;

    let mut subscription_id = 0;
    while let Some(msg) = ws_stream.next().await {
        match msg? {
            Message::Text(text) => {
                trace!("Received message: {} on client", text);
                match parse_message(&text.to_string()) {
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
                    StompMessage::Unsubscribe(us) => {
                        error!("Received unexpected unsubscribe message on client: {}", us.id);
                    },
                    StompMessage::Connect(ct) => {
                        error!("Received unexpected Connect message on client: {}", ct.accept_version);
                    },
                    StompMessage::Send(snd) => {
                        error!("Received unexpected Send message on client: {}", snd.destination);
                    },
                    StompMessage::Disconnect(_) => {
                        error!("Received unexpected Disconnect");
                        return Err(anyhow::anyhow!("Disconnected from client"));
                    },
                };
            }
            unexpected_message => {
                info!("Received unexpected non-text message on client: {:?}", unexpected_message);
            }
        }
    }
    Ok(())
}

fn unbox_handlers(handlers: Arc<RwLock<HashMap<String, Arc<dyn Fn(&MessageContent) + Send + Sync>>>>, unboxed_handlers: &mut HashMap<String, Arc<dyn Fn(&MessageContent) + Send + Sync>>) -> Result<()> {
    let readable_handlers = match handlers.read() {
        Ok(readable_handlers) => readable_handlers,
        Err(poison_error) => {
            return Err(anyhow::anyhow!("Could not to get readable handlers: {}", poison_error.to_string()));
        },
    };
    for (key, value) in readable_handlers.iter() {
        unboxed_handlers.insert(key.clone(), value.to_owned());
    }
    Ok(())
}

