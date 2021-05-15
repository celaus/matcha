use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, HashMap};
use xactor::*;
use anyhow::{Result, bail, ensure};
use async_trait::async_trait;
use crate::matching::Balance;

use super::matching::{Account, AccountId, OrderSide, Order, Transaction, Fill};

pub struct OrderBook {
  bids: BTreeMap<u64, Order>, 
  asks: BTreeMap<u64, Order>
}
#[message(result = "anyhow::Result<Vec<Account>>")]
pub enum AccountRequest {
  One(AccountId),
  All
}

pub enum AccountOperations {
  LockCollateral{ account: AccountId, balance: Balance }
}

#[message(result = "anyhow::Result<()>")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
  OrderIntent { account: AccountId, amount: usize, side: OrderSide, price: u64 } ,
  CancelIntent { account: AccountId, order_id: u64 }
}

pub struct AccountActor {
  accounts: HashMap<AccountId, Account>
}

#[async_trait]
impl Handler<Intent> for AccountActor {

    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: Intent) -> Result<()> {
      match msg {
          Intent::OrderIntent { account, amount, side, price } => {
            match self.accounts.get(&account) {
                Some(known_acct) if known_acct.free_collateral() >= amount => {
                  let order = Order::new(account, amount, side, price);
                  println!("order in: {:?}", order);
                  // if let Err(e) = Broker::from_registry().await?.publish(order) {
                  //     bail!("{}", e)
                  // } else {
                  //   Ok(())
                  // }
                  Ok(())
                }
                Some(known_acct) => bail!("Insufficient free collateral {} vs {}", known_acct.free_collateral(), amount),
                None =>  bail!("No account with id {}", account)
            }
          }
          Intent::CancelIntent { account, order_id } => 
            //Broker::from_registry().await?.publish(     
              Ok(())
      }
    }
}

#[async_trait]
impl Handler<AccountRequest> for AccountActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: AccountRequest) -> Result<Vec<Account>> {
      match msg {
          AccountRequest::One(account_id) => {
            ensure!(!self.accounts.contains_key(&account_id), "Account doesn't exist");
            Ok(vec![self.accounts.get(&account_id).unwrap().clone()])
          }
          AccountRequest::All => {
            Ok(self.accounts.values().cloned().collect())
          }
      }
      
    }
}

#[async_trait]
impl Handler<Account> for AccountActor {

    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: Account) -> Result<()> {
      ensure!(!self.accounts.contains_key(&msg.id), "Account already exists");
      self.accounts.insert(msg.id, msg);
      Ok(())
    }
}



#[async_trait]
impl Handler<Account> for AccountActor {

    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: Account) -> Result<()> {
      ensure!(!self.accounts.contains_key(&msg.id), "Account already exists");
      self.accounts.insert(msg.id, msg);
      Ok(())
    }
}

#[async_trait]
impl Actor for AccountActor {
    // async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
    //     ctx.subscribe::<Request>().await
    // }
}

///
/// Actor that downloads stock data for a specified symbol and period
///
pub struct OrderBookActor {
  pub order_book: OrderBook
}

#[async_trait]
impl Handler<Order> for OrderBookActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: Order) -> Vec<Fill> {
        match msg.side {
            OrderSide::Bid => {self.order_book.bids.insert(msg.price, msg);}
            OrderSide::Ask => {}
        }
        vec![]
    }
}

#[async_trait]
impl Actor for OrderBookActor {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
      Ok(())
    }
}
