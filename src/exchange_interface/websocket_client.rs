use std::sync::Arc;
use log::{debug, error, info};
use crate::config::BrokerConfig;
use crate::exchange_interface::trading::{MarketDepth, Execution, ExecutionsTopicWrapper, LastTrade, OrderState};
use crate::websockets;
use crate::websockets::client::StompMessage;

pub struct ExchangeWebsocketClient {
    pub config: BrokerConfig,
    pub execution_handler: fn(Execution),
    pub order_state_handler: fn(OrderState),
    pub depth_handler: fn(MarketDepth),
    pub last_trade_handler: fn(LastTrade),
}

impl ExchangeWebsocketClient {
    pub fn new(config: BrokerConfig,
               execution_handler: fn(Execution),
               order_state_handler: fn(OrderState),
               depth_handler: fn(MarketDepth),
               last_trade_handler: fn(LastTrade)) -> Self {
        ExchangeWebsocketClient {
            config,
            execution_handler,
            order_state_handler,
            depth_handler,
            last_trade_handler
        }}

    pub async fn start_exchange_websockets(&self) {
        let mut conn = websockets::client::WebsocketClient::new(&self.config);

        conn.subscribe("/user/queue/executions", build_executions_receiver(self.execution_handler, self.order_state_handler));
        conn.subscribe("/topics/depth", build_depth_receiver(self.depth_handler));
        conn.subscribe( "/topics/trades", build_last_trade_receiver(self.last_trade_handler));

        conn.start();
    }
}

fn build_executions_receiver(execution_handler: fn(Execution),
                             order_state_handler: fn(OrderState)) -> Arc<dyn Fn(&StompMessage) + Send + Sync + 'static> {
    Arc::new(move |message| executions_receiver( execution_handler, order_state_handler, message))
}

fn build_depth_receiver(depth_handler: fn(MarketDepth)) -> Arc<dyn Fn(&StompMessage) + Send + Sync + 'static> {
    Arc::new(move |message| depth_receiver( depth_handler, message))

}

fn build_last_trade_receiver(last_trade_handler: fn(LastTrade)) -> Arc<dyn Fn(&StompMessage) + Send + Sync + 'static> {
    Arc::new(move |message| last_trade_receiver( last_trade_handler, message))
}

fn executions_receiver(execution_handler: fn(Execution),
                       order_state_handler: fn(OrderState),
                       stomp_message: &StompMessage) {
    debug!("executions_receiver {} : '{}'", stomp_message.destination, stomp_message.body);

    let wrapper: ExecutionsTopicWrapper = match serde_json::from_str(stomp_message.body.as_str()) {
        Ok(x) => x,
        Err(y) => panic!("{}", y),
    };
    match wrapper.order_state {
        None => {}
        Some(order_state) => {
            (order_state_handler)(order_state);
        }
    }
    match wrapper.execution {
        None => {}
        Some(execution) => {
            (execution_handler)(execution);
        }
    }
}

fn depth_receiver(depth_handler: fn(MarketDepth), stomp_message: &StompMessage) {
    debug!("depth_receiver {} : {}", stomp_message.destination, stomp_message.body);
    let depth: MarketDepth = match serde_json::from_str(stomp_message.body.as_str()) {
        Ok(x) => x,
        Err(y) => panic!("{}", y),
    };
    (depth_handler)(depth);
}

fn last_trade_receiver(last_trade_handler: fn(LastTrade), stomp_message: &StompMessage) {
    debug!("trades_receiver {} : {}", stomp_message.destination, stomp_message.body);
    let last_trade: LastTrade = match serde_json::from_str(stomp_message.body.as_str()) {
        Ok(x) => x,
        Err(y) => panic!("{}", y),
    };
    (last_trade_handler)(last_trade);
}