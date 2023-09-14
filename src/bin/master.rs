#![feature(impl_trait_in_assoc_type)]

use std::net::SocketAddr;
use std::env;

use bdredis::{
    proxy2master::Proxy2MasterService,
    slave2master::Slave2MasterService,
};
use volo_gen::rds::{
    ScServiceServer,
    Slave2MasterServer, 
    Master2SlaveClient,
};

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;

use tracing::debug;
use tracing_subscriber::{fmt, util::SubscriberInitExt, prelude::__tracing_subscriber_SubscriberExt};

use tokio::signal;

#[volo::main]
async fn main() {
    tracing_subscriber::registry().with(fmt::layer()).init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("{:?}", args);
        panic!("Usage: master proxy_port master_port");
    }

    let db: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let slaves: Arc<Mutex<HashMap<String, Master2SlaveClient>>> = Arc::new(Mutex::new(HashMap::new()));

    // Proxy2Server RPC server
    let addr: SocketAddr = format!("127.0.0.1:{}", &args[1]).parse().unwrap();
    let addr = volo::net::Address::from(addr);
    tokio::task::spawn(ScServiceServer::new(Proxy2MasterService::new(db.clone(), slaves.clone())).run(addr));

    debug!("rpc server for proxy running at:{}", &args[1]);


    let addr: SocketAddr = format!("127.0.0.1:{}", &args[2]).parse().unwrap();
    let addr = volo::net::Address::from(addr);
    tokio::task::spawn(Slave2MasterServer::new(Slave2MasterService::new(slaves.clone())).run(addr));

    debug!("rpc server for slave running at:{}", &args[2]);

    match signal::ctrl_c().await {
        Ok(()) => {
        },
        Err(_) => {
        }
    }
}
