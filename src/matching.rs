use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use xactor::message;

use crate::actors::Intent;

static ORDERIDGEN: OnceCell<u64> = OnceCell::new();

fn get_order_id() -> u64 {
    let n = match ORDERIDGEN.get() {
        Some(i) => *i,
        None => 0,
    };

    ORDERIDGEN.set(n + 1);
    n
}

pub type AccountId = u64;
pub type Amount = usize;
pub type Balance = u64;
pub type Position = Order;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderSide {
    Bid,
    Ask,
}

#[message(result = "anyhow::Result<()>")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Action {
    Transaction {
        from: AccountId,
        to: AccountId,
        balance: Balance,
    },
    Fill {
        maker: Position,
        taker: Position,
    },
    Block {
        from: AccountId,
        balance: Balance,
    },
}

#[message(result = "anyhow::Result<()>")]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Account {
    pub id: AccountId,
    pub transactions: Vec<Action>,
}

impl Account {
    pub fn new() -> Self {
        Account {
            id: rand::random(),
            transactions: vec![],
        }
    }

    pub fn free_collateral(&self) -> Balance {
        let balance: i64 = self
            .transactions
            .iter()
            .map(|t| match t {
                Action::Transaction { from, balance, .. } => {
                    if from == &self.id {
                        *balance as i64 * -1
                    } else {
                        *balance as i64
                    }
                }
                Action::Fill { .. } => 0,
                Action::Block { .. } => 0,
            })
            .sum();

        if balance > 0 {
            balance as Balance
        } else {
            0
        }
    }
}

#[message(result = "anyhow::Result<Position>")]
pub struct ValidationMessage(pub Order);

#[message(result = "anyhow::Result<Vec<Action>>")]
pub struct ProcessPositionMessage(pub Position);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Order {
    pub id: u64,
    pub account: AccountId,
    pub amount: usize,
    pub version: usize,
    pub side: OrderSide,
    pub price: u64,
}

impl Order {
    pub fn new(account: AccountId, amount: usize, side: OrderSide, price: u64) -> Self {
        Order {
            id: get_order_id(),
            account,
            amount,
            version: 0,
            side,
            price,
        }
    }
    pub fn from_request(req: Intent) -> Option<Self> {
        match req {
            Intent::OrderIntent {
                account,
                amount,
                side,
                price,
            } => Some(Order {
                id: get_order_id(),
                account,
                amount,
                version: 0,
                side,
                price,
            }),
            _ => None,
        }
    }

    pub fn total(&self) -> u64 {
        self.price * self.amount as u64
    }
}
