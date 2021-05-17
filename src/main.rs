use std::collections::HashMap;
use xactor::*;
mod actors;
mod matching;
mod web;
use actors::*;

pub type ActorConnections = Connections;

#[derive(Clone)]
struct Connections {
    pub accounts: Addr<AccountActor>,
    pub orderbook: Addr<OrderBookActor>
}

#[xactor::main]
async fn main() -> Result<()> {
    env_logger::init();
    // Start actors. Supervisors also keep those actors alive
    let accounts = Supervisor::start(|| AccountActor {
        accounts: HashMap::new(),
    })
    .await?;
    let orderbook = Supervisor::start(|| OrderBookActor {
        order_book: OrderBook::new(),
    })
    .await?;

    let state = Connections { accounts, orderbook };

   web::start(state).await
}
