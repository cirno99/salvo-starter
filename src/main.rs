#[macro_use]
extern crate diesel;
extern crate dotenv;

#[macro_use]
extern crate serde;
#[macro_use]
extern crate bitflags;

#[macro_use]
mod macros;

#[macro_use]
extern crate diesel_migrations;

pub(crate) mod context;
pub(crate) mod data;
pub(crate) mod db;
pub(crate) mod email;
pub(crate) mod error;
pub(crate) mod helpers;
pub(crate) mod models;
pub(crate) mod routers;
pub(crate) mod schema;
pub(crate) mod things;
pub(crate) mod utils;

mod shared;

use dotenv::dotenv;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use tracing_futures::Instrument;

pub use error::Error;
pub use shared::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    user: i64,
    exp: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt().init();

    if dotenv::from_filename(".env.local").is_err() {
        println!("No .env.local file found, using .env file");
    }

    if let Err(e) = dotenv() {
        println!("Error loading .env file: {}", e);
    }

    println!("DATABASE_URL: {}", crate::database_url());
    tracing::info!(
        "=========================SAVVY APP STARTING======================================="
    );

    let mut build_result = db::build_pool(&crate::database_url());

    while let Err(e) = build_result {
        tracing::error!(error = ?e, "db connect failed, will try after 10 seconds...");
        std::thread::sleep(std::time::Duration::from_secs(10));
        build_result = db::build_pool(&crate::database_url());
    }
    if crate::db::DB_POOL.set(build_result.unwrap()).is_err() {
        tracing::error!("set db pool failed");
    } else {
        tracing::info!("db connected");
    }

    let mut conn = db::connect().unwrap();
    db::migrate(&mut conn);
    tracing::info!("db migrated");
    drop(conn);

    Server::new(TcpListener::new("0.0.0.0:7117").bind().await)
        .serve(routers::root())
        .instrument(tracing::info_span!("server.serve"))
        .await;
    Ok(())
}
