use crate::config::BrokerConfig;
use crate::exchange_interface::trading::{Execution, ExecutionsTopicWrapper, LastTrade, MarketDepth, OrderState};
use crate::persistence::dao::Dao;
use crate::websockets;
use crate::websockets::server::WebSocketServer;
use crate::websockets::stomp::MessageContent;
use log::{debug, error, info};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ExchangeWebsocketClient {
    pub config: BrokerConfig,
    pub dao: Dao,
    pub web_socket_server: WebSocketServer,
    pub execution_handler: fn(&Dao, &WebSocketServer, Execution) ,
    pub order_state_handler: fn(mutex: Arc<Mutex<()>>, &Dao, &WebSocketServer, OrderState),
    pub depth_handler: fn(&Dao, &WebSocketServer, MarketDepth),
    pub last_trade_handler: fn(&Dao, &WebSocketServer, LastTrade),
    pub mutex: Arc<Mutex<()>>,
}

impl ExchangeWebsocketClient {
    pub fn new(config: BrokerConfig,
               dao: Dao,
               web_socket_server: WebSocketServer,
               execution_handler: fn(&Dao, &WebSocketServer, Execution),
               order_state_handler: fn(mutex: Arc<Mutex<()>>, &Dao, &WebSocketServer, OrderState),
               depth_handler: fn(&Dao, &WebSocketServer, MarketDepth),
               last_trade_handler: fn(&Dao, &WebSocketServer, LastTrade)) -> Self {
        ExchangeWebsocketClient {
            config,
            dao,
            web_socket_server,
            execution_handler,
            order_state_handler,
            depth_handler,
            last_trade_handler,
            mutex: Arc::new(Mutex::new(())),
        }}

    pub async fn start_exchange_websockets(&self) {
        let mut conn = websockets::client::WebsocketClient::new(&self.config);

        conn.subscribe("/user/queue/executions", build_executions_receiver(self.mutex.clone(), self.dao.clone(), self.web_socket_server.clone(), self.execution_handler, self.order_state_handler));
        conn.subscribe("/topics/depth", build_depth_receiver(self.mutex.clone(), self.dao.clone(), self.web_socket_server.clone(), self.depth_handler));
        conn.subscribe( "/topics/trades", build_last_trade_receiver(self.mutex.clone(), self.dao.clone(), self.web_socket_server.clone(), self.last_trade_handler));

        conn.start();
    }
}

fn build_executions_receiver(mutex: Arc<Mutex<()>>, dao: Dao, web_socket_server: WebSocketServer, execution_handler: fn(&Dao, &WebSocketServer, Execution),
                             order_state_handler: fn(mutex: Arc<Mutex<()>>, &Dao, &WebSocketServer, OrderState)) -> Arc<dyn Fn(&MessageContent) + Send + Sync + 'static> {
    Arc::new(move |message| executions_receiver(mutex.clone(), &dao, &web_socket_server, execution_handler, order_state_handler, message))
}

fn build_depth_receiver(mutex: Arc<Mutex<()>>, dao: Dao, web_socket_server: WebSocketServer, depth_handler: fn(&Dao, &WebSocketServer, MarketDepth)) -> Arc<dyn Fn(&MessageContent)  + Send + Sync + 'static> {
    Arc::new(move |message| depth_receiver(mutex.clone(), &dao, &web_socket_server, depth_handler, message))

}

fn build_last_trade_receiver(mutex: Arc<Mutex<()>>, dao: Dao, web_socket_server: WebSocketServer, last_trade_handler: fn(&Dao, &WebSocketServer, LastTrade)) -> Arc<dyn Fn(&MessageContent) + Send + Sync + 'static> {
    Arc::new(move |message| last_trade_receiver(mutex.clone(), &dao, &web_socket_server, last_trade_handler, message))
}

fn executions_receiver(mutex: Arc<Mutex<()>>, dao: &Dao, web_socket_server: &WebSocketServer, execution_handler: fn(&Dao, &WebSocketServer, Execution),
                       order_state_handler: fn(mutex: Arc<Mutex<()>>, &Dao, &WebSocketServer, OrderState),
                       stomp_message: &MessageContent) {
    debug!("executions_receiver {} : '{}'", stomp_message.destination, stomp_message.body);

    let wrapper: ExecutionsTopicWrapper = match serde_json::from_str(stomp_message.body.as_str()) {
        Ok(x) => x,
        Err(y) => todo!("{}", y),
    };
    match wrapper.order_state {
        None => {}
        Some(order_state) => {
            (order_state_handler)(mutex, dao, web_socket_server, order_state);
        }
    }
    match wrapper.execution {
        None => {}
        Some(execution) => {
            (execution_handler)(dao, web_socket_server, execution);
        }
    }

}

fn depth_receiver(mutex: Arc<Mutex<()>>, dao: &Dao, web_socket_server: &WebSocketServer, depth_handler: fn(&Dao, &WebSocketServer, MarketDepth), stomp_message: &MessageContent) {
    debug!("depth_receiver {} : {}", stomp_message.destination, stomp_message.body);
    let depth: MarketDepth = match serde_json::from_str(stomp_message.body.as_str()) {
        Ok(x) => x,
        Err(y) => todo!("{}", y),
    };
    (depth_handler)(dao, web_socket_server, depth);
}

fn last_trade_receiver(mutex: Arc<Mutex<()>>, dao: &Dao, web_socket_server: &WebSocketServer, last_trade_handler: fn(&Dao, &WebSocketServer, LastTrade), stomp_message: &MessageContent) {
    debug!("trades_receiver {} : {}", stomp_message.destination, stomp_message.body);
    let last_trade: LastTrade = match serde_json::from_str(stomp_message.body.as_str()) {
        Ok(x) => x,
        Err(y) => todo!("{}", y),
    };
    (last_trade_handler)(dao, web_socket_server, last_trade);
}