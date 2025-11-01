use crate::access_control::AccessControl;
use crate::instrument_manager::InstrumentManager;
use crate::persistence::dao::Dao;
use crate::rest_api::account::{Account, Privilege};
use crate::rest_api::base_api;
use crate::websockets::client::StompMessage;
use crate::websockets::senders::{send_balance, send_orders, send_positions};
use crate::websockets::server::{QueueItem, WebSocketServer};
use crate::websockets::stomp;
use crate::websockets::stomp::{parse_message, SendContent, SubscribeContent};
use actix_web::web::ThinData;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::{AggregatedMessage, AggregatedMessageStream, Closed, Session};
use anyhow::Error;
use bimap::BiHashMap;
use futures_util::StreamExt;
use log::trace;
use log::{debug, error, info, warn};
use serde;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::spawn_local;
use tokio::{sync::mpsc, time::interval};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[get("/ws")]
pub async fn ws_setup(
    req: HttpRequest,
    dao: ThinData<Dao>,
    instrument_manager: ThinData<InstrumentManager>,
    session: actix_session::Session,
    web_socket_server: ThinData<WebSocketServer>,
    access_control: ThinData<AccessControl>,
    stream: web::Payload,
) -> HttpResponse {
    info!("Websocket connection requested for session {:p}", &session);

    let (res, ws_session, msg_stream) = match actix_ws::handle(&req, stream) {
        Ok(x) => x,
        Err(ws_error) => {
            error!("ws setup error {}", ws_error);
            return HttpResponse::InternalServerError().finish();
        },
    };

    let allowed_accounts = match access_control.get_allowed_accounts(&session) {
        Ok(allowed_accounts) => allowed_accounts,
        Err(_) => todo!(),
    };
    info!("allowed_accounts: {:?}", allowed_accounts);

    spawn_local(ws_handler(
        dao,
        instrument_manager,
        allowed_accounts,
        ws_session,
        web_socket_server,
        access_control,
        msg_stream
    ));

    res
}

async fn ws_handler(
    dao: ThinData<Dao>,
    instrument_manager: ThinData<InstrumentManager>,
    allowed_accounts: HashMap<String, Account>,
    mut ws_session: Session,
    web_socket_server: ThinData<WebSocketServer>,
    access_control: ThinData<AccessControl>,
    msg_stream: actix_ws::MessageStream,
) {
    let mut ws_handler_obj = WsHandler::new(dao, instrument_manager, web_socket_server, access_control, allowed_accounts, msg_stream);
    ws_handler_obj.start(&mut ws_session).await;
    info!("Websocket closing");
    match ws_session.close(None).await {
        Ok(_) => {},
        Err(closed) => {
            error!("Close error closing session {}", closed);
            return;
        },
    };
}

struct WsHandler {
    dao: ThinData<Dao>,
    instrument_manager: ThinData<InstrumentManager>,
    web_socket_server: ThinData<WebSocketServer>,
    access_control: ThinData<AccessControl>,
    allowed_accounts: HashMap<String, Account>,
    msg_stream: AggregatedMessageStream,
    subscriptions: BiHashMap<String, String>
}

impl WsHandler {
    fn new(dao: ThinData<Dao>,
           instrument_manager: ThinData<InstrumentManager>,
           web_socket_server: ThinData<WebSocketServer>,
           access_control: ThinData<AccessControl>,
           allowed_accounts: HashMap<String, Account>,
           in_msg_stream: actix_ws::MessageStream) -> WsHandler {
        WsHandler {
            dao,
            instrument_manager,
            web_socket_server,
            access_control,
            allowed_accounts,
            msg_stream:  in_msg_stream
                .max_frame_size(128 * 1024)
                .aggregate_continuations()
                .max_continuation_size(2 * 1024 * 1024),
            subscriptions: BiHashMap::new()
        }
    }

    async fn start(&mut self, ws_session: &mut Session) {
        info!("Websocket connected");
        let mut last_heartbeat = Instant::now();
        let mut interval = interval(HEARTBEAT_INTERVAL);

        let (conn_tx, mut conn_rx) = mpsc::unbounded_channel::<QueueItem>();

        loop {
            tokio::select! {
                Some(Ok(msg)) = self.msg_stream.next() => {
                    trace!("Received: {msg:?}");

                    match msg {
                        AggregatedMessage::Ping(bytes) => {
                            trace!("Websocket Ping");
                            last_heartbeat = Instant::now();
                            match ws_session.pong(&bytes).await {
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
                            match self.parse_text_message(ws_session, &conn_tx, &text.to_string()).await {
                                Ok(_) => {},
                                Err(closed) => {
                                    error!("Could not send text, exiting, due to {}", closed);
                                    return;
                                },
                            };
                        }
                        AggregatedMessage::Binary(_bin) => {
                            warn!("Unexpected binary message");
                        }
                        AggregatedMessage::Close(reason) => {
                            info!("Close message: {:?}", reason);
                            self.unsubscribe_all(&conn_tx);
                            return
                        },
                    }
                }
                Some(queue_item) = conn_rx.recv() => {
                    let  send_result = self.send_queue_item(ws_session, queue_item).await;
                    match send_result {
                        Ok(_) => {},
                        Err(closed) => {
                            error!("Could not send text for queued_item, exiting, due to {}", closed);
                            return;
                        },
                    };
                }
                _ = interval.tick() => {
                    if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                        info!("Websocket client timeout");
                        return;
                    }
                    let _ = ws_session.ping(b"").await;
                }
                else => {
                    return;
                }
            }
        };
    }
    async fn send_queue_item(&mut self, session: &mut Session, queue_item: QueueItem) -> Result<(), Closed> {
        let subscription_id_options = self.subscriptions.get_by_left(&queue_item.destination);
        let subscription_id = match subscription_id_options {
            Some(subscription_id) => subscription_id,
            None => {
                debug!("No subscription for {}", queue_item.destination);
                return Ok(());
            },
        };
        let data_message = stomp::text_message(queue_item.destination, subscription_id.clone(), &queue_item.body);
        let data_message_string = data_message.to_string();
        trace!("Sending {}", data_message_string);
        session.text(data_message_string).await
    }


    async fn validate_subscription(&self, destination: &String) -> bool {
        let account_key_result = extract_account_key(destination);
        let account_key = match account_key_result {
            Ok(account_key) => account_key,
            Err(broker_error) => {
                error!("{}", broker_error);
                return false;
            }
        };
        let allowed: bool = match self.access_control.is_allowed_from_map(&self.allowed_accounts, &account_key, Privilege::Read) {
            Ok(allowed) => allowed,
            Err(error) => {
                error!("Failed while checking access: {}", error.to_string());
                return false;
            }
        };
        allowed
    }


    fn unsubscribe_all(&mut self, conn_tx: &UnboundedSender<QueueItem>) {
        let mut ids_to_remove = Vec::new();
        let immutable_subscriptions = self.subscriptions.clone();
        {
            for right_val in immutable_subscriptions.right_values() {
                ids_to_remove.push(right_val);
            }
        }
        for id_to_remove in ids_to_remove {
            self.unsubscribe(&conn_tx, id_to_remove.clone());
        }
    }

    fn unsubscribe(&mut self, conn_tx: &UnboundedSender<QueueItem>, id: String) {
        let destination = match self.subscriptions.remove_by_right(&id) {
            Some(destination) => destination,
            None => {
                warn!("Received Unsubscribe message on server for unknown subscription: {}", id);
                return;
            },
        };
        let mut writable_conns = match self.web_socket_server.connections.write() {
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


    async fn send_content(&self, conn_tx: UnboundedSender<QueueItem>, content: SendContent)  -> Result<(), Closed> {
        if !self.validate_subscription(&content.destination).await {
            error!("Request to send for forbidden destination {} {}", content.destination, content.body);
            return Err(Closed);
        }

        let dao_clone = self.dao.clone();
        let instrument_manager_clone = self.instrument_manager.clone();

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
                        send_get(dao_clone, instrument_manager_clone, conn_tx, &content.destination, &account_key, send_request.scope).await
                    }
                };
            }
        );
        Ok(())
    }

    async fn parse_text_message(&mut self, session: &mut Session, conn_tx: &UnboundedSender<QueueItem>,
                                text: &String) -> Result<(), Error> {
        debug!("Text message {}", text);
        let parsed_message_result = parse_message(&text.to_string());
        let parsed_message = match parsed_message_result {
            Ok(parsed_message) => parsed_message,
            Err(parse_error) => return Err(anyhow::anyhow!("Unable to parse message: {}", parse_error.to_string()))
        };
        let res = match parsed_message {
            StompMessage::Message(msg) => {
                error!("Received unexpected Message message on server: {}", msg.body);
                Ok(())
            }
            StompMessage::Send(msg) => {
                info!("Received expected Send message on server: {} {}", msg.destination, msg.body );
                self.send_content(conn_tx.clone(), msg).await
            }
            StompMessage::Connected(ct) => {
                error!("Received unexpected Connected message on server: {}", ct.user_name);
                Ok(())
            }
            StompMessage::Subscribe(sub) => {
                info!("Received expected Subscribe message on server: {} as {}", sub.destination, sub.id);
                self.handle_subscribe(conn_tx, &sub).await
            },
            StompMessage::Connect(ct) => {
                info!("Received expected Connect message on server: {}", ct.accept_version);
                 session.text(stomp::connected_message().to_string()).await
            },
            StompMessage::Unsubscribe(us) => {
                info!("Received expected Unsubscribe message on server: {}", us.id);
                self.unsubscribe(&conn_tx, us.id);
                Ok(())
            },
            StompMessage::Disconnect(_) => {
                info!("Received expected Disconnect message on server");
                self.unsubscribe_all(&conn_tx);
                Ok(())
            }
        };
        match res {
            Ok(_) => Ok(()),
            Err(closed_error) => {
                Err(anyhow::Error::from(closed_error))
            }
        }
    }

    async fn handle_subscribe(&mut self, conn_tx: &UnboundedSender<QueueItem>, sub: &SubscribeContent) -> Result<(), Closed> {
        if sub.destination.starts_with("/accounts/") {
            if !self.validate_subscription(&sub.destination).await {
                error!("Request for forbidden destination {}", sub.destination);
                return Err(Closed);
            }
        }
        let mut writable_conns = match self.web_socket_server.connections.write() {
            Ok(writable_conns) => writable_conns,
            Err(poison_error) => {
                error!("Subscribe message could not get writable_conns {}", poison_error.to_string());
                return Ok(());
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
                error!("No per_destination_conns for {}", sub.destination);
                return Ok(());
            },
        };
        self.subscriptions.insert(sub.destination.clone(), sub.id.clone());
        let readable = match self.web_socket_server.retained_messages.read() {
            Ok(readable) => readable,
            Err(readable_error) => {
                error!("Unable to get read access to retained_messages: {}", readable_error);
                return Ok(());
            }
        };
        match readable.get(&sub.destination) {
            None => {}
            Some(retained_message) => {
                conn_tx.send(retained_message.clone()).unwrap_or_else(|send_error| error!("Error when sending retained message: {}", send_error));
            }
        };
        Ok(())
    }
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


fn extract_account_key(destination: &String) -> Result<String, anyhow::Error> {
    let path_elements = destination.split("/").collect::<Vec<&str>>();
    let path_length = path_elements.len();
    if path_length != 4 {
        return Err(anyhow::anyhow!("/account path {} has {} elements, not expected 4", destination, path_length));
    }
    let account_key = path_elements[2].to_string();
    Ok(account_key)
}

async fn send_get(dao: ThinData<Dao>, instrument_manager: ThinData<InstrumentManager>, conn_tx: UnboundedSender<QueueItem>, destination: &String, account_key: &String, scope: Scope) {
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
            send_positions(txn, &instrument_manager, conn_tx, destination, account_key).await;
        }
        Scope::Orders => {
            send_orders(txn, &instrument_manager, conn_tx, destination, account_key).await;
        }
    };
}