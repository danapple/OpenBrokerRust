use futures_util::StreamExt;
use log::{debug, error, info};
use serde;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use strfmt::strfmt;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Debug)]
pub struct WebSocketServer {
    pub connections: Arc<RwLock<HashMap<String, Vec<UnboundedSender<QueueItem>>>>>
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
        let queue_item = match Self::create_queue_item(&destination, body) {
            Ok(value) => value,
            Err(queue_item_error) => {
                error!("send_message queue item error {}", queue_item_error.to_string());
                return;
            },
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

    fn create_queue_item(destination: &String, body: &impl Serialize) -> Result<QueueItem, anyhow::Error> {
        let serialized_body = match serde_json::to_string(body) {
            Ok(x) => x,
            Err(fmt_error) => {
                return Err(anyhow::anyhow!("send_message serialization error {}", fmt_error.to_string()));
            },
        };
        info!("send_message to destination {}: {}", destination, serialized_body);
        let queue_item = QueueItem {
            destination: destination.clone(),
            body: serialized_body
        };
        Ok(queue_item)
    }
}
