use std::{cell::RefCell, sync::{atomic::{AtomicU64, Ordering}}};

use once_cell::sync::OnceCell;
use serde::{Deserialize,Serialize};
use xactor::message;

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
  Bid, Ask
}


#[message(result = "anyhow::Result<()>")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Action {
    Transaction {
    from: AccountId, 
    to: AccountId, 
    balance: Balance
    }, 
    Fill {
      maker: Position, 
      taker: Position
    },
    Block {
      from: AccountId,
      balance: Balance
  }
}

#[message(result = "anyhow::Result<()>")]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Account {
  pub id: AccountId,
  pub transactions: Vec<Action>
}

impl Account {
  pub fn new() -> Self {
    Account {
      id: rand::random(),
      transactions: vec![]
    }
  }

  pub fn free_collateral(&self) -> Balance {
    let balance: isize = self.transactions.iter().map(|t| if t.from == self.id {
      t.balance as isize * -1
    } else {
      t.balance as isize
    }).sum();
    
    if balance > 0 {
      balance as Balance
    }
    else {
      0
    }
  }
}



#[message(result = "Vec<Fill>")]
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
      price
  }
  }
  pub fn from_request(req: Request) -> Option<Self> {
        match req {
            Request::OrderIntent { account, amount, side, price } => {
              Some(Order {
                  id: get_order_id(),
                  account,
                  amount,
                  version: 0,
                  side,
                  price,
                
              })}
            _ => None
        }
    }
}
