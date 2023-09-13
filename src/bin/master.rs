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

use tokio::fs::File;
use tokio::io::AsyncReadExt;
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
    if args.len() != 2 {
        panic!("Usage: master proxy_port");
    }

    let db: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let slaves: Arc<Mutex<HashMap<String, Master2SlaveClient>>> = Arc::new(Mutex::new(HashMap::new()));

    // Proxy2Server RPC server
    let addr: SocketAddr = format!("127.0.0.1:{}", &args[1]).parse().unwrap();
    let addr = volo::net::Address::from(addr);
    tokio::task::spawn(ScServiceServer::new(Proxy2MasterService::new(db.clone(), slaves.clone())).run(addr));

    debug!("rpc server for proxy running at:{}", &args[1]);


    // 读取Master-Slave配置文件，获取master server端口号
    let conf_path = "../../config/ms.conf";
    let mut conf_file = File::open(conf_path).await.unwrap();

    debug!("read configure");

    let mut lines = String::new();
    let _ = conf_file.read_to_string(&mut lines).await;
    let master_addr = String::from(lines.split_whitespace().next().unwrap());
    let master_addr: SocketAddr = master_addr.parse().unwrap();
    let addr = volo::net::Address::from(master_addr);
    tokio::task::spawn(Slave2MasterServer::new(Slave2MasterService::new(slaves.clone())).run(addr));

    match signal::ctrl_c().await {
        Ok(()) => {
        },
        Err(_) => {
        }
    }
}
