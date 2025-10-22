use crate::access_control::{AccessControl, Privilege};
use crate::websockets::client::StompMessage;
use crate::websockets::stomp;
use crate::websockets::stomp::parse_message;
use actix_web::web::ThinData;
use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt;
use log::{debug, error, info, warn};
use serde;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use strfmt::strfmt;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::spawn_local;
use tokio::{sync::mpsc, time::interval};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Debug)]
pub struct WebSocketServer {
    connections: Arc<RwLock<HashMap<String, Vec<UnboundedSender<QueueItem>>>>>
}

#[derive(Serialize, Clone, Debug)]
struct QueueItem {
    destination: String,
    body: String
}

impl WebSocketServer {
    pub fn new() -> Self {
        WebSocketServer{
            connections: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    pub fn send_account_message(&mut self, account_key: String, destination: &str, body: &impl Serialize) {
        let mut vars = HashMap::new();
        vars.insert("account_key".to_string(), account_key);
        let new_destination = strfmt(destination, &vars).unwrap();
        self.send_message(new_destination, body);
    }
    pub fn send_message(&mut self, destination: String, body: &impl Serialize) {
        let serialized_body = serde_json::to_string(body).unwrap();
        info!("send_message: {} : {}", destination, serialized_body);
        let queue_item = QueueItem{
            destination: destination.clone(),
            body: serialized_body
        };
        match self.connections.write() {
            Ok(mut writable_conns) => {
                let conns_list_wrapped = writable_conns.get_mut(&destination);
                match conns_list_wrapped {
                    None => {
                        debug!("No subscribers for {}", destination);
                    }
                    Some(conns_list) => {
                        let mut dropped_conns = Vec::new();
                        for (pos, conn) in conns_list.iter().enumerate() {
                            match conn.send(queue_item.clone()) {
                                Ok(x) => x,
                                Err(y) => {
                                    error!("Connection had error '{}'", y.to_string());
                                    dropped_conns.push(pos);
                                },
                            };
                        }
                        dropped_conns.reverse();
                        for pos in dropped_conns {
                            conns_list.remove(pos);
                        }
                    }
                }
            },
            Err(_) => todo!(),
        };
    }
}


pub async fn ws_setup(
    req: HttpRequest,
    web_socket_server: ThinData<WebSocketServer>,
    access_control: ThinData<AccessControl>,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {

    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

    let customer_key = match req.cookie("customerKey") {
        Some(x) => x,
        None => todo!("No customer key available"),
    }.value().to_string();
    if customer_key.is_empty() {
        return Err(error::ErrorBadRequest("customerKey is empty"));
    }

    info!("Websocket connection established for {}", customer_key);
    spawn_local(ws_handler(
        session,
        web_socket_server,
        access_control,
        msg_stream,
        customer_key
    ));

    Ok(res)
}

async fn ws_handler(
    mut session: actix_ws::Session,
    web_socket_server: ThinData<WebSocketServer>,
    access_control: ThinData<AccessControl>,
    msg_stream: actix_ws::MessageStream,
    customer_key: String,
) {
    info!("Websocket connected for {}", customer_key);
    let mut last_heartbeat = Instant::now();
    let mut interval = interval(HEARTBEAT_INTERVAL);

    let mut msg_stream = msg_stream
        .max_frame_size(128 * 1024)
        .aggregate_continuations()
        .max_continuation_size(2 * 1024 * 1024);

    let mut subscriptions = HashMap::new();

    let (conn_tx, mut conn_rx) = mpsc::unbounded_channel::<QueueItem>();

    let close_reason = loop {
        tokio::select! {
            Some(Ok(msg)) = msg_stream.next() => {
                info!("msg: {msg:?}");

                match msg {
                    AggregatedMessage::Ping(bytes) => {
                        info!("Websocket Ping");
                        last_heartbeat = Instant::now();
                        session.pong(&bytes).await.unwrap();
                    }

                    AggregatedMessage::Pong(_) => {
                        info!("Websocket Pong");
                        last_heartbeat = Instant::now();
                    }

                    AggregatedMessage::Text(text) => {
                        info!("Text message {}", text);
                        match parse_message(&text.to_string()) {
                            StompMessage::Message(msg) => {
                                error!("Received unexpected Message message on server: {}", msg.body);
                            }
                            StompMessage::Connected(ct) => {
                                error!("Received unexpected Connected message on server: {}", ct.user_name);
                            }
                            StompMessage::Subscribe(sub) => {
                                info!("Received expected Subscribe message on server: {} as {}", sub.destination, sub.id);

                                if sub.destination.starts_with("/account/") {
                                    validate_subscription(&access_control, &sub.destination, &customer_key)
                                }
                                match web_socket_server.connections.write() {
                                    Ok(mut writable_conns) => {
                                        if !writable_conns.contains_key(&sub.destination) {
                                            writable_conns.insert(sub.destination.clone(), Vec::new());
                                        }
                                        match writable_conns.get_mut(&sub.destination) {
                                            Some(per_destination_conns) => {
                                                per_destination_conns.push(conn_tx.clone());
                                            },
                                            None => todo!(),
                                        };
                                    },
                                    Err(_) => todo!(),
                                };

                                subscriptions.insert(sub.destination.clone(), sub.id);

                            },
                            StompMessage::Connect(ct) => {
                                info!("Received expected Connect message on server: {}", ct.accept_version);
                                session.text(stomp::connected_message().to_string()).await.unwrap();
                            }
                        };
    // }
                    }
                    AggregatedMessage::Binary(_bin) => {
                        warn!("Unexpected binary message");
                    }
                    AggregatedMessage::Close(reason) => {
                        info!("Close message: {:?}", reason);
                        break reason
                    },
                }
            }
            Some(queue_item) = conn_rx.recv() => {
                let subscription_id = subscriptions.get(&queue_item.destination);
                match subscription_id {
                    Some(id) => {
                        let data_message = stomp::data_message(queue_item.destination, id.clone(), &queue_item.body);
                        let data_message_string = data_message.to_string();
                        info!("Sending {}", data_message_string);
                        session.text(data_message_string).await.unwrap();
                    }
                    None => {
                        debug!("No subscription for {}", queue_item.destination);
                    },
                }
            }
            _ = interval.tick() => {
                if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                    info!("Websocket client timeout");
                    break None;
                }
                let _ = session.ping(b"").await;
            }

            else => {
                break None;
            }
        }
    };
    info!("Websocket closing");

    let _ = session.close(close_reason).await;
}

fn validate_subscription(access_control: &AccessControl, destination: &String, customer_key: &String) {
    let path_elements = destination.split("/").collect::<Vec<&str>>();
    let path_length = path_elements.len();
    if path_length != 4 {
        todo!("/account path {} has {} elements, not expected 4", destination, path_length);
    }
    let account_key = path_elements[2].to_string();
    if !access_control.is_allowed(&account_key, &customer_key, Privilege::Read) {
        todo!("Customer not allowed");
    }
}
