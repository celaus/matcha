use async_std::prelude::*;
use async_std::stream;
use chrono::prelude::*;
use clap::Clap;
use matching::{Account, AccountId, Order};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use std::time::Duration;
use xactor::*;
mod actors;
mod matching;
use actors::*;
use tide::{Body, Request, Response, StatusCode};

type ActorConnections = Connections;

#[derive(Clone)]
struct Connections {
    pub accounts: Addr<AccountActor>
}
#[derive(Clap)]
#[clap(
    version = "1.0",
    author = "Claus Matzinger",
    about = "A Manning LiveProject: async Rust"
)]
struct Opts {
    #[clap(short, long, default_value = "AAPL,MSFT,UBER,GOOG")]
    symbols: String,
    #[clap(short, long)]
    from: String,
}

async fn accounts_list(req: Request<ActorConnections>) -> tide::Result {
    let mut response_builder = Response::new(StatusCode::Ok);

    let data: Vec<Account> = {
        let conns = req.state();
        conns.accounts.call(AccountRequest::All).await??
    };
    response_builder.set_body(Body::from_json(&data)?);
    Ok(response_builder)
}

async fn account_new(mut req: Request<ActorConnections>) -> tide::Result {
    let account: Account = req.body_json().await?;
    let created: Result<()> = {
        let conns = req.state();
        conns.accounts.call(account.clone()).await?
    };
    let (status, body) = match created {
        Ok(_) => {(StatusCode::Ok, Body::from_json(&account)?)}
        Err(e) => {
            (StatusCode::BadRequest, Body::from_json(&json!({ "error": e.to_string() }))?)
        }
    };

    let mut response_builder = Response::new(status);
    response_builder.set_body(body);
    Ok(response_builder)
}

async fn account_by_id(req: Request<ActorConnections>) -> tide::Result {
    let id: AccountId = req.param("id")?.parse()?;
    let data: Account = {
        let conns = req.state();
        conns.accounts.call(AccountRequest::One(id)).await??.into_iter().next().unwrap()
    };

    let mut response_builder = Response::new(StatusCode::Ok);
    response_builder.set_body(Body::from_json(&data)?);
    Ok(response_builder)
}


async fn orders_list(req: Request<ActorConnections>) -> tide::Result {
    let mut response_builder = Response::new(StatusCode::Ok);
    let data = ();
    response_builder.set_body(Body::from_json(&data)?);
    Ok(response_builder)
}

async fn order_new(mut req: Request<ActorConnections>) -> tide::Result {
    let order: Order = req.body_json().await?;
    let data = ();
    let mut response_builder = Response::new(StatusCode::Ok);
    response_builder.set_body(Body::from_json(&data)?);
    Ok(response_builder)
}

async fn order_by_id(req: Request<ActorConnections>) -> tide::Result {
    let n: usize = req.param("id")?.parse()?;
    let data = ();
    let mut response_builder = Response::new(StatusCode::Ok);
    response_builder.set_body(Body::from_json(&data)?);
    Ok(response_builder)
}


async fn order_delete(req: Request<ActorConnections>) -> tide::Result {
    let n: usize = req.param("id")?.parse()?;
    let data = ();
    let mut response_builder = Response::new(StatusCode::Ok);
    response_builder.set_body(Body::from_json(&data)?);
    Ok(response_builder)
}

async fn orderbook_show(req: Request<ActorConnections>) -> tide::Result {
    let n: usize = req.param("id")?.parse()?;
    let data = ();
    let mut response_builder = Response::new(StatusCode::Ok);
    response_builder.set_body(Body::from_json(&data)?);
    Ok(response_builder)
}

///
/// Main!
///
#[xactor::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let from: DateTime<Utc> = opts.from.parse().expect("Couldn't parse 'from' date");
    let symbols: Vec<String> = opts.symbols.split(',').map(|s| s.to_owned()).collect();

    // Start actors. Supervisors also keep those actors alive
    let accounts = Supervisor::start(|| AccountActor {accounts: HashMap::new()}).await?;
    let state = Connections {
        accounts
    };
    let mut app = tide::with_state(state.clone());
    // Schedule HTTP server task "in background"
    let _http_endpoint = async_std::task::spawn(async move {

        app.at("/accounts").nest({
            let mut api = tide::with_state(state.clone());
            api.at("/").get(accounts_list);
            api.at("/").put(account_new);
            api.at("/:id").get(account_by_id);
            api
        });
        app.at("/orders").nest({
            let mut api = tide::with_state(state.clone());
            api.at("/").get(orders_list);
            api.at("/").put(order_new);
            api.at("/:id").get(order_by_id);
            api.at("/:id").delete(order_delete);
            api
        });
        app.at("/orderbook").get(orderbook_show);

        app.listen("localhost:4321").await
    });


    Ok(())
}
