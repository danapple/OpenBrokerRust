use crate::constants::APPLICATION_JSON;
use anyhow::anyhow;
use log::error;
use std::collections::HashMap;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

pub fn connect_message() -> Message {
    Message::text("CONNECT\naccept-version:1.2,2.0\n\n\x00")
}

pub fn connected_message() -> Message {
    Message::text("CONNECTED\nversion:1.2\n\n\x00")
}

pub fn subscribe_message(subscription_id: u32, destination: &str) -> Message {
    Message::text(format!("SUBSCRIBE\nid:{}\ndestination:{}\nack:auto\n\n\x00", subscription_id, destination))
}

pub fn text_message(destination: String, subscription: String, body: &String) -> Message {
    Message::text(format!("\
    MESSAGE\n\
    destination:{}\n\
    content_type:{}\n\
    subscription:{}\n\
    message_id:{}\n\
    content_length:{}\n\n\
    {}\n\
    \n\x00",
    destination,
    APPLICATION_JSON,
        subscription,
        Uuid::new_v4().simple().to_string(),
        body.len(),
        body))
}

pub enum StompMessage {
    Message(MessageContent),
    Send(SendContent),
    Connect(ConnectContent),
    Connected(ConnectedContent),
    Subscribe(SubscribeContent),
    Unsubscribe(UnsubscribeContent),
    Disconnect(DisconnectContent)
}

pub struct MessageContent {
    pub destination: String,
    pub content_type: String,
    pub subscription: u16,
    pub message_id: String,
    pub content_length: u16,
    pub body: String,
}

#[derive(Debug)]
pub struct SendContent {
    pub destination: String,
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
    pub id: String,
    pub destination: String,
    pub ack: String,
}

pub struct UnsubscribeContent {
    pub id: String,
}

pub struct DisconnectContent {
}

const MESSAGE: &'static str = "MESSAGE";
const SEND: &'static str = "SEND";
const CONNECT: &'static str = "CONNECT";
const CONNECTED: &'static str = "CONNECTED";
const SUBSCRIBE: &'static str = "SUBSCRIBE";
const UNSUBSCRIBE: &'static str = "UNSUBSCRIBE";
const DISCONNECT: &'static str = "DISCONNECT";

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
const MESSAGE_TYPE: &'static str = "message-type";
const BODY: &'static str = "body";

pub fn parse_message(message: &String) -> Result<StompMessage, anyhow::Error> {
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
    let message_type = match vals.get(MESSAGE_TYPE) {
        Some(message_type) => message_type,
        None => return Err(anyhow!("Missing message type")),
    };
    let ret = match message_type.as_str() {
        MESSAGE => {
            let content_length = match vals[CONTENT_LENGTH].parse::<u16>() {
                Ok(content_length) => content_length,
                Err(parse_error) =>
                    return Err(anyhow!("Could not parse content length {}: {}", vals[CONTENT_LENGTH], parse_error.to_string())),
            };
            let subscription = match vals[SUBSCRIPTION].parse::<u16>() {
                Ok(subscription) => subscription,
                Err(parse_error) =>
                    return Err(anyhow!("Could not parse subscription {}: {}", vals[SUBSCRIPTION], parse_error.to_string())),
            };
            StompMessage::Message(MessageContent {
                destination: vals[DESTINATION].to_string(),
                body: vals[BODY].to_string(),
                content_type: vals[CONTENT_TYPE].to_string(),
                content_length,
                subscription,
                message_id: vals[MESSAGE_ID].to_string(),
            })
        },
        SEND => {
            StompMessage::Send(SendContent {
                destination: vals[DESTINATION].to_string(),
                body: vals[BODY].to_string(),
            })
        },
        CONNECT => {
            StompMessage::Connect(ConnectContent {
                accept_version: vals[ACCEPT_VERSION].to_string(),
            })
        },
        CONNECTED => {
            StompMessage::Connected(ConnectedContent {
                version: vals[VERSION].to_string(),
                heart_beat: vals[HEART_BEAT].to_string(),
                user_name: vals[USER_NAME].to_string(),

            })
        },
        SUBSCRIBE => {
            StompMessage::Subscribe(SubscribeContent {
                id: vals[ID].to_string(),
                destination: vals[DESTINATION].to_string(),
                ack: match vals.get(ACK) {
                    Some(x) => x,
                    None => "auto",
                }.to_string(),
            })
        },
        UNSUBSCRIBE => {
            StompMessage::Unsubscribe(UnsubscribeContent {
                id: vals[ID].to_string()
            })
        }
        DISCONNECT => {
            StompMessage::Disconnect(DisconnectContent {
            })
        }
        _ => {
            error!("Unknown message type {}", message_type);
            return Err(anyhow::anyhow!("Unknown message type {}", message_type));
        }
    };
    Ok(ret)
}
