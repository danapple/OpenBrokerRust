use std::collections::HashMap;
use log::info;
use tokio_tungstenite::tungstenite::Message;

pub fn connect() -> Message {
    Message::Text("CONNECT\naccept-version:1.2,2.0\n\n\x00".to_string())
}

pub fn subscribe_message(subscription_id: u32, destination: &str) -> Message {
    Message::Text(format!("SUBSCRIBE\nid:{}\ndestination:{}\nack:auto\n\n\x00", subscription_id, destination))
}

pub fn parse_message(message: &String) -> StompMessage {
    let mut vals = HashMap::new();
    let mut body_now = false;
    for line in message.lines() {
        let line = line.trim().trim_matches(char::from(0));
        if line.eq("MESSAGE") {
            continue
        }
        else if line.eq("") {
            body_now = true;
        }
        else if body_now {
            vals.insert("body", line.to_string());
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
    StompMessage {
        destination: vals["destination"].to_string(),
        body: vals["body"].to_string(),
        content_type: vals["content-type"].to_string(),
        content_length: vals["content-length"].parse::<u16>().unwrap(),
        subscription: vals["subscription"].parse::<u16>().unwrap(),
        message_id: vals["message-id"].to_string(),
    }
}
pub struct StompMessage {
    pub destination: String,
    pub content_type: String,
    pub subscription: u16,
    pub message_id: String,
    pub content_length: u16,
    pub body: String,
}