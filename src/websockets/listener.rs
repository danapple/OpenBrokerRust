use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use anyhow::Result;
use log::info;
use tokio::net::TcpListener;
use crate::config::BrokerConfig;

pub async fn start_websocket_listener(config: &BrokerConfig) -> Result<()> {
    let addr = config.websocket_addr.to_string();
    let websocket_listener = TcpListener::bind(&addr).await?;
    info!("WebSocket bound to ws://{}", addr);

    tokio::spawn(run_websocket_listener(websocket_listener));

    Ok(())

}
pub async fn run_websocket_listener(websocket_listener: TcpListener) {
    info!("WebSocket listener started");
    while let Ok((stream, _)) = websocket_listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}
async fn handle_connection(stream: tokio::net::TcpStream) -> Result<()> {
    info!("WebSocket connection established");

    let mut ws_stream = accept_async(stream).await?;

    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        if msg.is_text() {
            let received_text = msg.to_text()?;
            info!("Received message: {}", received_text);

            ws_stream.send(Message::Text(received_text.to_string())).await?;
        }
    }

    Ok(())
}