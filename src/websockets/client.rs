use crate::exchange_interface::exchange_client::get_customer_key_cookie;
use crate::websockets::stomp;
pub(crate) use crate::websockets::stomp::StompMessage;
use crate::websockets::stomp::{parse_message, MessageContent};
use anyhow::Result;
use async_std::task;
use futures_util::SinkExt;
use log::{debug, error, info};
use log::{trace, warn};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

pub struct WebsocketClient {
    websocket_address: String,
    customer_key: String,
    handlers: Arc<RwLock<HashMap<String, Arc<dyn Fn(&MessageContent) + Send + Sync + 'static>>>>,
}

impl WebsocketClient {
    pub fn new<'e>(websocket_address: String, customer_key: String) -> Self {
        WebsocketClient {
            websocket_address,
            customer_key,
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn subscribe(&mut self, destination: &str, func: Arc<dyn Fn(&MessageContent) + Send + Sync + 'static>){
        info!("Requesting subscribe to {}", destination);
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
        let f = run_websocket(self.websocket_address.clone(), self.customer_key.clone(), self.handlers.clone());
        tokio::spawn(f);
    }
}

async fn run_websocket(websocket_address: String, broker_key: String, handlers: Arc<RwLock<HashMap<String, Arc<dyn Fn(&MessageContent) + Send + Sync + 'static>>>>) -> Result<()> {
    let mut unboxed_handlers = HashMap::new();
    match unbox_handlers(handlers, &mut unboxed_handlers) {
        Ok(_) => {},
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
            return Err(anyhow::anyhow!("Could parse api key cookie: {}", parse_error)),
    };
    request.headers_mut().insert("Cookie", customer_key_cookie);
    let five_seconds = time::Duration::from_millis(5000);

    loop {
        run_one_web_socket(request.clone(), &unboxed_handlers).await;
        task::sleep(five_seconds).await;
    }
}

pub async fn run_one_web_socket(request: Request, unboxed_handlers: &HashMap<String, Arc<dyn Fn(&MessageContent) + Send + Sync>>) {
    let (mut ws_stream, _) = match connect_async(request).await {
        Ok(x) => x,
        Err(connect_error) => {
            error!("Unable to connect to endpoint: {}", connect_error.to_string());
            return;
        },
    };
    println!("WebSocket client connected");
    match ws_stream.send(stomp::connect_message()).await {
        Ok(_) => {},
        Err(connect_error) => {
            error!("Unable to send connect message: {}", connect_error.to_string());
            return;
        },
    };

    let mut subscription_id = 0;

    loop {
        let msg_raw = ws_stream.next().await;
        let msg_result = match msg_raw {
            Some(msg_option) => msg_option,
            None => {
                error!("End of web socket stream");
                return;
            }
        };
        let msg = match msg_result {
            Ok(msg) => msg,
            Err(msg_error) => {
                error!("Message error: {}", msg_error.to_string());
                return;
            }
        };
        subscription_id = match process_message(&mut ws_stream, unboxed_handlers, msg, subscription_id).await {
            Ok(subscription_id) => subscription_id,
            Err(process_error) => {
                error!("Process error: {}", process_error.to_string());
                return;
            }
        };
    }
}

async fn process_message(ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, unboxed_handlers: &HashMap<String, Arc<dyn Fn(&MessageContent) + Send + Sync>>, msg: Message, subscription_id: u32) -> Result<u32> {
    let mut new_subscription_id = subscription_id;
    match msg {
        Message::Text(text) => {
            debug!("Received message: {} on client", text);
            let parsed_message_result = parse_message(&text.to_string());
            let parsed_message = match parsed_message_result {
                Ok(parsed_message) => parsed_message,
                Err(parse_error) => {
                    return Err(anyhow::anyhow!("Unable to parse message: {}", parse_error.to_string()));
                }
            };
            match parsed_message {
                StompMessage::Message(msg) => {
                    debug!("Handling StompMessage:Message");
                    match unboxed_handlers.get(msg.destination.as_str()) {
                        Some(func) => func(&msg),
                        _ => {}
                    }
                }
                StompMessage::Connected(_) => {
                    debug!("Received expected Connected message on client");
                    for (destination, _) in unboxed_handlers.iter() {
                        info!("Subscribing to {} with subscription id {}", destination, new_subscription_id);
                        match ws_stream.send(stomp::subscribe_message(new_subscription_id, destination)).await {
                            Ok(_) => {},
                            Err(send_error) => {
                                return Err(anyhow::anyhow!("Send error: {}", send_error.to_string()));
                            }
                        };
                        new_subscription_id += 1
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
                    return Err(anyhow::anyhow!("Received unexpected Disconnect"));
                },
            };
        }
        unexpected_message => {
            warn!("Received unexpected non-text message on client: {:?}", unexpected_message);
        }
    }
    Ok(new_subscription_id)
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

