use crate::config::BrokerConfig;
use crate::exchange_interface::market_data::{LastTrade, MarketDepth};
use crate::exchange_interface::order::{Execution, ExecutionsTopicWrapper, OrderState};
use crate::instrument_manager::InstrumentManager;
use crate::persistence::dao::Dao;
use crate::websockets;
use crate::websockets::server::WebSocketServer;
use crate::websockets::stomp::MessageContent;
use log::{debug, error, info};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ExchangeWebsocketClient {
    pub websocket_address: String, 
    pub customer_key: String,
    pub dao: Dao,
    pub web_socket_server: WebSocketServer,
    pub instrument_manager: InstrumentManager,
    pub execution_handler: fn(mutex: Arc<Mutex<()>>, &Dao, &WebSocketServer, &InstrumentManager, Execution) ,
    pub order_state_handler: fn(mutex: Arc<Mutex<()>>, &Dao, &WebSocketServer, &InstrumentManager, OrderState),
    pub depth_handler: fn(&WebSocketServer, &InstrumentManager, MarketDepth),
    pub last_trade_handler: fn(&WebSocketServer, &InstrumentManager, LastTrade),
    pub mutex: Arc<Mutex<()>>,
}

impl ExchangeWebsocketClient {
    pub fn new(websocket_address: String, 
               customer_key: String,
               dao: Dao,
               web_socket_server: WebSocketServer,
               instrument_manager: InstrumentManager,
               execution_handler: fn(mutex: Arc<Mutex<()>>, &Dao, &WebSocketServer, &InstrumentManager, Execution),
               order_state_handler: fn(mutex: Arc<Mutex<()>>, &Dao, &WebSocketServer, &InstrumentManager, OrderState),
               depth_handler: fn(&WebSocketServer, &InstrumentManager, MarketDepth),
               last_trade_handler: fn(&WebSocketServer, &InstrumentManager, LastTrade)) -> Self {
        ExchangeWebsocketClient {
            websocket_address,
            customer_key,
            dao,
            web_socket_server,
            instrument_manager,
            execution_handler,
            order_state_handler,
            depth_handler,
            last_trade_handler,
            mutex: Arc::new(Mutex::new(())),
        }}

    pub async fn start_exchange_websockets(&self) {
        let mut conn = websockets::client::WebsocketClient::new(self.websocket_address.clone(), self.customer_key.clone());

        conn.subscribe("/user/queue/executions", build_executions_receiver(self.mutex.clone(), self.dao.clone(), self.web_socket_server.clone(), self.instrument_manager.clone(), self.execution_handler, self.order_state_handler));
        conn.subscribe("/topics/depth", build_depth_receiver(self.mutex.clone(), self.web_socket_server.clone(), self.instrument_manager.clone(), self.depth_handler));
        conn.subscribe( "/topics/trades", build_last_trade_receiver(self.mutex.clone(), self.web_socket_server.clone(), self.instrument_manager.clone(), self.last_trade_handler));

        conn.start();
    }
}

fn build_executions_receiver(mutex: Arc<Mutex<()>>, dao: Dao, web_socket_server: WebSocketServer, instrument_manager: InstrumentManager, execution_handler: fn(mutex: Arc<Mutex<()>>, &Dao, &WebSocketServer, &InstrumentManager, Execution),
                             order_state_handler: fn(mutex: Arc<Mutex<()>>, &Dao, &WebSocketServer, &InstrumentManager, OrderState)) -> Arc<dyn Fn(&MessageContent) + Send + Sync + 'static> {
    Arc::new(move |message| executions_receiver(mutex.clone(), &dao, &web_socket_server, &instrument_manager,  execution_handler, order_state_handler, message))
}

fn build_depth_receiver(mutex: Arc<Mutex<()>>, web_socket_server: WebSocketServer, instrument_manager: InstrumentManager, depth_handler: fn(&WebSocketServer, &InstrumentManager, MarketDepth)) -> Arc<dyn Fn(&MessageContent)  + Send + Sync + 'static> {
    Arc::new(move |message| depth_receiver(mutex.clone(), &web_socket_server, &instrument_manager, depth_handler, message))
}

fn build_last_trade_receiver(mutex: Arc<Mutex<()>>, web_socket_server: WebSocketServer, instrument_manager: InstrumentManager, last_trade_handler: fn(&WebSocketServer, &InstrumentManager, LastTrade)) -> Arc<dyn Fn(&MessageContent) + Send + Sync + 'static> {
    Arc::new(move |message| last_trade_receiver(mutex.clone(), &web_socket_server, &instrument_manager, last_trade_handler, message))
}

fn executions_receiver(mutex: Arc<Mutex<()>>, dao: &Dao, web_socket_server: &WebSocketServer, instrument_manager: &InstrumentManager,
                       execution_handler: fn(mutex: Arc<Mutex<()>>, &Dao, &WebSocketServer, &InstrumentManager, Execution),
                       order_state_handler: fn(mutex: Arc<Mutex<()>>, &Dao, &WebSocketServer, &InstrumentManager, OrderState),
                       stomp_message: &MessageContent) {
    debug!("executions_receiver {} : '{}'", stomp_message.destination, stomp_message.body);

    let wrapper: ExecutionsTopicWrapper = match serde_json::from_str(stomp_message.body.as_str()) {
        Ok(x) => x,
        Err(y) => {
            error!("Failed while parsing executions message: {}", y);
            return;
        },
    };
    match wrapper.order_state {
        None => {}
        Some(order_state) => {
            order_state_handler(mutex.clone(), dao, web_socket_server, instrument_manager, order_state);
        }
    }
    match wrapper.execution {
        None => {}
        Some(execution) => {
            execution_handler(mutex, dao, web_socket_server, instrument_manager, execution);
        }
    }

}

fn depth_receiver(mutex: Arc<Mutex<()>>, web_socket_server: &WebSocketServer, instrument_manager: &InstrumentManager, depth_handler: fn(&WebSocketServer, &InstrumentManager, MarketDepth), stomp_message: &MessageContent) {
    debug!("depth_receiver {} : {}", stomp_message.destination, stomp_message.body);
    let depth: MarketDepth = match serde_json::from_str(stomp_message.body.as_str()) {
        Ok(x) => x,
        Err(y) => {
            error!("Failed while parsing depth message: {}", y);
            return;
        },
    };
    depth_handler(web_socket_server, instrument_manager, depth);
}

fn last_trade_receiver(mutex: Arc<Mutex<()>>, web_socket_server: &WebSocketServer, instrument_manager: &InstrumentManager, last_trade_handler: fn(&WebSocketServer, &InstrumentManager, LastTrade), stomp_message: &MessageContent) {
    debug!("trades_receiver {} : {}", stomp_message.destination, stomp_message.body);
    let last_trade: LastTrade = match serde_json::from_str(stomp_message.body.as_str()) {
        Ok(x) => x,
        Err(y) => {
            error!("Failed while parsing last_trade message: {}", y);
            return;
        },
    };
    last_trade_handler(web_socket_server, instrument_manager, last_trade);
}