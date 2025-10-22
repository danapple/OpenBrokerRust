use crate::constants::APPLICATION_JSON;
use log::{error, info};
use serde::Serialize;
use std::collections::HashMap;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

pub fn connect_message() -> Message {
    Message::Text("CONNECT\naccept-version:1.2,2.0\n\n\x00".to_string())
}

pub fn subscribe_message(subscription_id: u32, destination: &str) -> Message {
    Message::Text(format!("SUBSCRIBE\nid:{}\ndestination:{}\nack:auto\n\n\x00", subscription_id, destination))
}

pub fn data_message(destination: String, subscription: u16, thing: &impl Serialize) -> Message{
    let string = match serde_json::to_string(thing) {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    text_message(destination,
                 subscription,
                 Uuid::new_v4().simple().to_string(),
                 string)
}
fn text_message(destination: String, subscription: u16, message_id: String, body: String) -> Message {
    Message::Text(format!("\
    MESSAGE\n\
    destination:{}\n\
    content_type:{}\n\
    subscription:{}\n\
    message_id:{}\n\
    content_length:{}\n\
    body:{}\n\
    \n\x00",
    destination,
    APPLICATION_JSON,
        subscription,
        message_id,
        body.len(),
        body))
}

pub enum StompMessage {
    Message(MessageContent),
    Connect(ConnectContent),
    Connected(ConnectedContent),
    Subscribe(SubscribeContent)
}

pub struct MessageContent {
    pub destination: String,
    pub content_type: String,
    pub subscription: u16,
    pub message_id: String,
    pub content_length: u16,
    pub body: String,
}

pub struct ConnectedContent {
    pub version: String,
    pub heart_beat: String,
    pub user_name: String,
}

pub struct ConnectContent {
    pub accept_version: String,
}

pub struct SubscribeContent {
    pub id: u16,
    pub destination: String,
    pub ack: String,
}

const MESSAGE_TYPE: &'static str = "message-type";
const BODY: &'static str = "body";
const MESSAGE: &'static str = "MESSAGE";
const DESTINATION: &'static str = "destination";
const CONTENT_TYPE: &'static str = "content-type";
const CONTENT_LENGTH: &'static str = "content-length";
const SUBSCRIPTION: &'static str = "subscription";
const MESSAGE_ID: &'static str = "message-id";
const ACCEPT_VERSION: &'static str = "accept-version";
const VERSION: &'static str = "version";
const HEART_BEAT: &'static str = "heart-beat";
const USER_NAME: &'static str = "user-name";
const ID: &'static str = "id";
const ACK: &'static str = "ack";

pub fn parse_message(message: &String) -> StompMessage {
    let mut vals: HashMap<&str, String> = HashMap::new();
    let mut body_now = false;
    for line in message.lines() {
        let line = line.trim().trim_matches(char::from(0));
        if !vals.contains_key(MESSAGE_TYPE) {
            vals.insert(MESSAGE_TYPE, line.to_string());
        }
        else if line.len() == 0 {
            body_now = true;
        }
        else if body_now {
            vals.insert(BODY, line.to_string());
        }
        else {
            let parts = line.split_once(":");
            match parts {
                None => {}
                Some((key, value)) => {
                  //  info!("key: {} value: {}", key, value);
                    vals.insert(key, value.to_string());
                }
            }
        }
    }
    let message_type = vals.get(MESSAGE_TYPE).unwrap();
    let ret = match message_type.as_str() {
        MESSAGE => {
            StompMessage::Message(MessageContent {
                destination: vals[DESTINATION].to_string(),
                body: vals[BODY].to_string(),
                content_type: vals[CONTENT_TYPE].to_string(),
                content_length: vals[CONTENT_LENGTH].parse::<u16>().unwrap(),
                subscription: vals[SUBSCRIPTION].parse::<u16>().unwrap(),
                message_id: vals[MESSAGE_ID].to_string(),
            })
        },
        "CONNECT" => {
            StompMessage::Connect(ConnectContent {
                accept_version: vals[ACCEPT_VERSION].to_string(),
            })
        },
        "CONNECTED" => {
            StompMessage::Connected(ConnectedContent {
                version: vals[VERSION].to_string(),
                heart_beat: vals[HEART_BEAT].to_string(),
                user_name: vals[USER_NAME].to_string(),

            })
        },
        "SUBSCRIBE" => {
            StompMessage::Subscribe(SubscribeContent {
                id: vals[ID].parse::<u16>().unwrap(),
                destination: vals[DESTINATION].to_string(),
                ack: vals[ACK].to_string(),
            })

        }
        _ => {
            error!("Unknown message type {}", message_type);
            todo!()}
    };
    ret
}
