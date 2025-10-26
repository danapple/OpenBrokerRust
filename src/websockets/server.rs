use crate::access_control::{AccessControl, Privilege};
use crate::errors::BrokerError;
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
use std::sync::{Arc, LockResult, RwLock};
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
        let new_destination = match strfmt(destination, &vars) {
            Ok(new_destination) => new_destination,
            Err(fmt_error) => {
                error!("send_account_message strfmt error {}", fmt_error.to_string());
                return;
            },
        };
        self.send_message(new_destination, body);
    }
    pub fn send_message(&mut self, destination: String, body: &impl Serialize) {
        let serialized_body = match serde_json::to_string(body) {
            Ok(x) => x,
            Err(fmt_error) => {
                error!("send_message serialization error {}", fmt_error.to_string());
                return;
            },
        };
        info!("send_message: {} : {}", destination, serialized_body);
        let queue_item = QueueItem{
            destination: destination.clone(),
            body: serialized_body
        };
        let mut writable_conns = match self.connections.write() {
            Ok(writable_conns) => writable_conns,
            Err(poison_error) => {
                error!("send_message could not get writable_conns {}", poison_error.to_string());
                return;
            },
        };
        let conns_list_wrapped = writable_conns.get_mut(&destination);
        let conns_list = match conns_list_wrapped {
            None => {
                debug!("No subscribers for {}", destination);
                return;
            }
            Some(conns_list) => conns_list
        };
        let mut dropped_conns = Vec::new();
        for (pos, conn) in conns_list.iter().enumerate() {
            match conn.send(queue_item.clone()) {
                Ok(_) => {},
                Err(send_error) => {
                    error!("Connection had error '{}', dropping", send_error.to_string());
                    dropped_conns.push(pos);
                }
            };
        }
        dropped_conns.reverse();
        for pos in dropped_conns {
            conns_list.remove(pos);
        }
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
        Some(customer_key) => customer_key,
        None => {
            error!("No customer key available");
            return Err(error::ErrorBadRequest("No customer key available".to_string()));
        }
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
                        match session.pong(&bytes).await {
                            Ok(_) => {},
                            Err(closed) => {
                                error!("Ping error while sending pong {}", closed);
                                return;
                            },
                        };
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
                                        match session.clone().close(None).await {
                                            Ok(_) => {},
                                            Err(closed) => {
                                                error!("Close error closing session {}", closed);
                                                return;
                                            },
                                        };
                                    }
                                }
                                let mut writable_conns = match web_socket_server.connections.write() {
                                    Ok(writable_conns) => writable_conns,
                                    Err(poison_error) => {
                                        error!("Subscribe message could not get writable_conns {}", poison_error.to_string());
                                        return;
                                    },
                                };
                                if !writable_conns.contains_key(&sub.destination) {
                                    writable_conns.insert(sub.destination.clone(), Vec::new());
                                }
                                match writable_conns.get_mut(&sub.destination) {
                                    Some(per_destination_conns) => {
                                        per_destination_conns.push(conn_tx.clone());
                                    },
                                    None => {
                                        error!("Not per_destination_conns for {}", sub.destination);
                                        return;
                                    },
                                };
                                subscriptions.insert(sub.destination.clone(), sub.id);
                            },
                            StompMessage::Connect(ct) => {
                                info!("Received expected Connect message on server: {}", ct.accept_version);
                                match session.text(stomp::connected_message().to_string()).await {
                                    Ok(_) => {},
                                    Err(closed) => {
                                        error!("Could not send text {}", closed);
                                        match session.close(None).await {
                                            Ok(_) => {},
                                            Err(closed) => {
                                                error!("Close error closing session {}", closed);
                                                return;
                                            },
                                        };
                                        return;
                                    },
                                };
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
                let subscription_id_options = subscriptions.get_by_left(&queue_item.destination);
                let subscription_id = match subscription_id_options {
                    Some(subscription_id) => subscription_id,
                    None => {
                        debug!("No subscription for {}", queue_item.destination);
                        return;
                    },
                };
                let data_message = stomp::text_message(queue_item.destination, subscription_id.clone(), &queue_item.body);
                let data_message_string = data_message.to_string();
                trace!("Sending {}", data_message_string);
                match session.text(data_message_string).await {
                    Ok(x) => x,
                    Err(closed) => {
                        error!("Could not send text for queued_item {}", closed);
                        match session.close(None).await {
                            Ok(_) => {},
                            Err(closed) => {
                                error!("Close error closing session {}", closed);
                                return;
                            },
                        };
                        return;
                    },
                };
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
            let send_request: SendRequest = match serde_json::from_str(content.body.as_str()) {
                Ok(send_request) => send_request,
                Err(serde_error) => {
                    error!("send_content deserialization error {}", serde_error.to_string());
                    return;
                }
            };
            let account_key_result = extract_account_key(&content.destination);
            let account_key = match account_key_result {
                Ok(account_key) => account_key,
                Err(broker_error) => {
                    error!("{}", broker_error);
                    return;
                }
            };
            match send_request.request {
                Request::GET => {
                    send_get(dao, conn_tx, &content.destination, &account_key, send_request.scope).await
                }
            };
        }
    );
}

async fn send_get(dao: ThinData<Dao>, conn_tx: UnboundedSender<QueueItem>, destination: &String, account_key: &String, scope: Scope) {
    let mut db_connection = match dao.get_connection().await {
        Ok(db_connection) => db_connection,
        Err(dao_error) => {
            error!("Unable to get_connection {}", dao_error);
            return;
        },
    };
    let txn = match dao.begin(&mut db_connection).await {
        Ok(txn) => txn,
        Err(dao_error) => {
            error!("Unable to get_connection {}", dao_error);
            return;
        },
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
    let account_key_result = extract_account_key(destination);
    let account_key = match account_key_result {
        Ok(account_key) => account_key,
        Err(broker_error) => {
            error!("{}", broker_error);
            return false;
        }
    };
    access_control.is_allowed(&account_key, Some(customer_key.to_string()), Privilege::Read).await
}

fn extract_account_key(destination: &String) -> Result<String, BrokerError> {
    let path_elements = destination.split("/").collect::<Vec<&str>>();
    let path_length = path_elements.len();
    if path_length != 4 {
        return Err(BrokerError::failure(format!("/account path {} has {} elements, not expected 4", destination, path_length)));
    }
    let account_key = path_elements[2].to_string();
    Ok(account_key)
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
    let destination = match subscriptions.remove_by_right(&id) {
        Some(destination) => destination,
        None => {
            warn!("Received Unsubscribe message on server for unknown subscription: {}", id);
            return;
        },
    };
    let mut writable_conns = match web_socket_server.connections.write() {
        Ok(writable_conns) => writable_conns,
        Err(poison_error) => {
            error!("unsubscribe could not get writable_conns {}", poison_error.to_string());
            return;
        }
    };

    let per_destination_conns = match writable_conns.get_mut(destination.0.as_str()) {
        Some(per_destination_conns) => per_destination_conns,
        _ => {
            return;
        }
    };

    let index = per_destination_conns.iter().position(|this_conn| conn_tx.same_channel(this_conn));
    match index {
        Some(ind) => {
            per_destination_conns.remove(ind);
            debug!("Unsubscribed here {}", ind);
        },
        None => {
            warn!("Cannot find connection to remove for unsubscribe: {}:{}", destination.0, destination.1);
        }
    };
}
