use std::pin::Pin;
use std::sync::{Arc};
use tokio::sync::Mutex;
use log::{debug, error, info};
use crate::config::BrokerConfig;
use crate::exchange_interface::trading::{MarketDepth, Execution, ExecutionsTopicWrapper, LastTrade, OrderState};
use crate::persistence::dao::Dao;
use crate::websockets;
use crate::websockets::client::StompMessage;

pub struct ExchangeWebsocketClient {
    pub config: BrokerConfig,
    pub dao: Dao,
    pub execution_handler: fn(&Dao, Execution) ,
    pub order_state_handler: fn(mutex: Arc<Mutex<()>>, &Dao, OrderState),
    pub depth_handler: fn(&Dao, MarketDepth),
    pub last_trade_handler: fn(&Dao, LastTrade),
    pub mutex: Arc<Mutex<()>>,
}

impl ExchangeWebsocketClient {
    pub fn new(config: BrokerConfig,
               dao: Dao,
               execution_handler: fn(&Dao, Execution),
               order_state_handler: fn(mutex: Arc<Mutex<()>>, &Dao, OrderState),
               depth_handler: fn(&Dao, MarketDepth),
               last_trade_handler: fn(&Dao, LastTrade)) -> Self {
        ExchangeWebsocketClient {
            config,
            dao,
            execution_handler,
            order_state_handler,
            depth_handler,
            last_trade_handler,
            mutex: Arc::new(Mutex::new(())),
        }}

    pub async fn start_exchange_websockets(&self) {
        let mut conn = websockets::client::WebsocketClient::new(&self.config);

        conn.subscribe("/user/queue/executions", build_executions_receiver(self.mutex.clone(), self.dao.clone(), self.execution_handler, self.order_state_handler));
        conn.subscribe("/topics/depth", build_depth_receiver(self.mutex.clone(), self.dao.clone(), self.depth_handler));
        conn.subscribe( "/topics/trades", build_last_trade_receiver(self.mutex.clone(), self.dao.clone(), self.last_trade_handler));

        conn.start();
    }
}

fn build_executions_receiver(mutex: Arc<Mutex<()>>, dao: Dao, execution_handler: fn(&Dao, Execution),
                             order_state_handler: fn(mutex: Arc<Mutex<()>>, &Dao, OrderState)) -> Arc<dyn Fn(&StompMessage) + Send + Sync + 'static> {
    Arc::new(move |message| executions_receiver(mutex.clone(), &dao, execution_handler, order_state_handler, message))
}

fn build_depth_receiver(mutex: Arc<Mutex<()>>, dao: Dao, depth_handler: fn(&Dao, MarketDepth)) -> Arc<dyn Fn(&StompMessage)  + Send + Sync + 'static> {
    Arc::new(move |message| depth_receiver(mutex.clone(), &dao, depth_handler, message))

}

fn build_last_trade_receiver(mutex: Arc<Mutex<()>>, dao: Dao, last_trade_handler: fn(&Dao, LastTrade)) -> Arc<dyn Fn(&StompMessage) + Send + Sync + 'static> {
    Arc::new(move |message| last_trade_receiver(mutex.clone(), &dao, last_trade_handler, message))
}

fn executions_receiver(mutex: Arc<Mutex<()>>, dao: &Dao, execution_handler: fn(&Dao, Execution),
                       order_state_handler: fn(mutex: Arc<Mutex<()>>, &Dao, OrderState),
                       stomp_message: &StompMessage) {
    debug!("executions_receiver {} : '{}'", stomp_message.destination, stomp_message.body);

    let wrapper: ExecutionsTopicWrapper = match serde_json::from_str(stomp_message.body.as_str()) {
        Ok(x) => x,
        Err(y) => panic!("{}", y),
    };
    match wrapper.order_state {
        None => {}
        Some(order_state) => {
            (order_state_handler)(mutex, dao, order_state);
        }
    }
    match wrapper.execution {
        None => {}
        Some(execution) => {
            (execution_handler)(dao, execution);
        }
    }

}

fn depth_receiver(mutex: Arc<Mutex<()>>, dao: &Dao, depth_handler: fn(&Dao, MarketDepth), stomp_message: &StompMessage) {
    debug!("depth_receiver {} : {}", stomp_message.destination, stomp_message.body);
    let depth: MarketDepth = match serde_json::from_str(stomp_message.body.as_str()) {
        Ok(x) => x,
        Err(y) => panic!("{}", y),
    };
    (depth_handler)(dao, depth);
}

fn last_trade_receiver(mutex: Arc<Mutex<()>>, dao: &Dao, last_trade_handler: fn(&Dao, LastTrade), stomp_message: &StompMessage) {
    debug!("trades_receiver {} : {}", stomp_message.destination, stomp_message.body);
    let last_trade: LastTrade = match serde_json::from_str(stomp_message.body.as_str()) {
        Ok(x) => x,
        Err(y) => panic!("{}", y),
    };
    (last_trade_handler)(dao, last_trade);
}