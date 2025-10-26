use crate::access_control::{AccessControl, Privilege};
use crate::persistence::dao::Dao;
use crate::rest_api::base_api;
use crate::websockets::client::StompMessage;
use crate::websockets::senders::{send_balance, send_orders, send_positions};
use crate::websockets::stomp;
use crate::websockets::stomp::{parse_message, SendContent};
use actix_web::web::ThinData;
use actix_web::{error, web, HttpRequest, HttpResponse};
use actix_ws::AggregatedMessage;
use bimap::{BiHashMap, BiMap};
use futures_util::StreamExt;
use log::trace;
use log::{debug, error, info, warn};
use serde;
use serde::{Deserialize, Serialize};
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
pub(crate) struct QueueItem {
    pub(crate) destination: String,
    pub(crate) body: String
}

impl WebSocketServer {
    pub fn new() -> Self {
        WebSocketServer{
            connections: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    pub fn send_account_message(&mut self, account_key: &str, destination: &str, body: &impl Serialize) {
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
    dao: ThinData<Dao>,
    web_socket_server: ThinData<WebSocketServer>,
    access_control: ThinData<AccessControl>,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {

    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

    let customer_key = base_api::get_customer_key(req);
    let customer_key = match customer_key {
        Some(x) => x,
        None => todo!("No customer key available"),
    }.to_string();
    if customer_key.is_empty() {
        return Err(error::ErrorBadRequest("customerKey is empty"));
    }

    info!("Websocket connection established for {}", customer_key);
    spawn_local(ws_handler(
        dao,
        session,
        web_socket_server,
        access_control,
        msg_stream,
        customer_key
    ));

    Ok(res)
}

async fn ws_handler(
    dao: ThinData<Dao>,
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

    let mut subscriptions = BiMap::new();

    let (conn_tx, mut conn_rx) = mpsc::unbounded_channel::<QueueItem>();

    let close_reason = loop {
        tokio::select! {
            Some(Ok(msg)) = msg_stream.next() => {
                trace!("Received: {msg:?}");

                match msg {
                    AggregatedMessage::Ping(bytes) => {
                        trace!("Websocket Ping");
                        last_heartbeat = Instant::now();
                        session.pong(&bytes).await.unwrap();
                    }

                    AggregatedMessage::Pong(_) => {
                        trace!("Websocket Pong");
                        last_heartbeat = Instant::now();
                    }

                    AggregatedMessage::Text(text) => {
                        debug!("Text message {}", text);
                        match parse_message(&text.to_string()) {
                            StompMessage::Message(msg) => {
                                error!("Received unexpected Message message on server: {}", msg.body);
                            }
                            StompMessage::Send(msg) => {
                                info!("Received expected Send message on server: {} {}", msg.destination, msg.body );
                                send_content(dao.clone(), &access_control, conn_tx.clone(), &customer_key, msg).await;
                            }
                            StompMessage::Connected(ct) => {
                                error!("Received unexpected Connected message on server: {}", ct.user_name);
                            }
                            StompMessage::Subscribe(sub) => {
                                info!("Received expected Subscribe message on server: {} as {}", sub.destination, sub.id);

                                if sub.destination.starts_with("/accounts/") {
                                    if !validate_subscription(&access_control, &sub.destination, &customer_key).await {
                                        error!("Request for forbidden destination {}", sub.destination);
                                        session.clone().close(None).await.unwrap();
                                    }
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
                            },
                            StompMessage::Unsubscribe(us) => {
                                info!("Received expected Unsubscribe message on server: {}", us.id);
                                unsubscribe(&web_socket_server, &mut subscriptions, &conn_tx, us.id);
                            },
                            StompMessage::Disconnect(_) => {
                                info!("Received expected Disconnect message on server");
                                unsubscribe_all(&web_socket_server, &mut subscriptions, &conn_tx);
                            }
                        };
                    }
                    AggregatedMessage::Binary(_bin) => {
                        warn!("Unexpected binary message");
                    }
                    AggregatedMessage::Close(reason) => {
                        info!("Close message: {:?}", reason);
                        unsubscribe_all(&web_socket_server, &mut subscriptions, &conn_tx);
                        break reason
                    },
                }
            }
            Some(queue_item) = conn_rx.recv() => {
                let subscription_id = subscriptions.get_by_left(&queue_item.destination);
                match subscription_id {
                    Some(id) => {
                        let data_message = stomp::text_message(queue_item.destination, id.clone(), &queue_item.body);
                        let data_message_string = data_message.to_string();
                        trace!("Sending {}", data_message_string);
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

#[derive(Debug, Deserialize)]
enum Request {
    GET
}

#[derive(Debug, Deserialize)]
enum Scope {
    #[serde(rename = "balance")]
    Balance,
    #[serde(rename = "positions")]
    Positions,
    #[serde(rename = "orders")]
    Orders,
}

#[derive(Debug, Deserialize)]
struct SendRequest {
    pub request: Request,
    pub scope: Scope,
}
async fn send_content(dao: ThinData<Dao>, access_control: &ThinData<AccessControl>,
                      conn_tx: UnboundedSender<QueueItem>, customer_key: &String, content: SendContent) {
    if !validate_subscription(&access_control, &content.destination, &customer_key).await {
        error!("Request to send for forbidden destination {} {}", content.destination, content.body);
        return;
    }

    tokio::spawn(async move
        {
            let mess: SendRequest = serde_json::from_str(content.body.as_str()).unwrap();
            let account_key = extract_account_key(&content.destination);

            match mess.request {
                Request::GET => {
                    send_get(dao, conn_tx, &content.destination, &account_key, mess.scope).await
                }
            };
        }
    );
}

async fn send_get(dao: ThinData<Dao>, conn_tx: UnboundedSender<QueueItem>, destination: &String, account_key: &String, scope: Scope) {
    let mut db_connection = match dao.get_connection().await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    let txn = match dao.begin(&mut db_connection).await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    match scope {
        Scope::Balance => {
            send_balance(txn, conn_tx, destination, account_key).await;
        }
        Scope::Positions => {
            send_positions(txn, conn_tx, destination, account_key).await;
        }
        Scope::Orders => {
            send_orders(txn, conn_tx, destination, account_key).await;
        }
    };
}

async fn validate_subscription(access_control: &AccessControl, destination: &String, customer_key: &String) -> bool {
    let account_key = extract_account_key(destination);
    access_control.is_allowed(&account_key, Some(customer_key.to_string()), Privilege::Read).await
}

fn extract_account_key(destination: &String) -> String {
    let path_elements = destination.split("/").collect::<Vec<&str>>();
    let path_length = path_elements.len();
    if path_length != 4 {
        todo!("/account path {} has {} elements, not expected 4", destination, path_length);
    }
    let account_key = path_elements[2].to_string();
    account_key
}

fn unsubscribe_all(web_socket_server: &ThinData<WebSocketServer>, subscriptions: &mut BiHashMap<String, String>, conn_tx: &UnboundedSender<QueueItem>) {
    let mut ids_to_remove = Vec::new();
    let immutable_subscriptions = subscriptions.clone();
    {
        for right_val in immutable_subscriptions.right_values() {
            ids_to_remove.push(right_val);
        }
    }
    for id_to_remove in ids_to_remove {
        unsubscribe(&web_socket_server, subscriptions, &conn_tx, id_to_remove.clone());
    }}
fn unsubscribe(web_socket_server: &ThinData<WebSocketServer>, subscriptions: &mut BiHashMap<String, String>, conn_tx: &UnboundedSender<QueueItem>, id: String) {
    match subscriptions.remove_by_right(&id) {
        Some(destination) => {
            match web_socket_server.connections.write() {
                Ok(mut writable_conns) => {
                    match writable_conns.get_mut(destination.0.as_str()) {
                        Some(per_destination_conns) => {
                            let index = per_destination_conns.iter().position(|this_conn| conn_tx.same_channel(this_conn));
                            match index {
                                Some(ind) => {
                                    per_destination_conns.remove(ind);
                                    info!("Unsubscribed here {}", ind);
                                },
                                None => {
                                    warn!("Cannot find connection to remove for unsubscribe: {}:{}", destination.0, destination.1);
                                }
                            };

                        },
                        None => {
                            warn!("Cannot find connections for unsubscribe: {}:{}", destination.0, destination.1);
                        },
                    };
                },
                Err(_) => todo!(),
            };
        },
        None => {
            warn!("Received Unsubscribe message on server for unknown subscription: {}", id);
        },
    };
}
