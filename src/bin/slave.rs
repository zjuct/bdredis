#![feature(impl_trait_in_assoc_type)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::env;

use bdredis::{
    proxy2slave::Proxy2SlaveService,
    master2slave::Master2SlaveService,
};
use pilota::FastStr;
use tracing::{debug, error};
use volo_gen::rds::{
    ScServiceServer,
    Slave2MasterClient, Slave2MasterClientBuilder,
    Master2SlaveServer,
};

use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;

use std::sync::Arc;

use tracing_subscriber::{fmt, util::SubscriberInitExt, prelude::__tracing_subscriber_SubscriberExt};

use volo_gen::rds::PingRequest;
use tokio::signal;


#[volo::main]
async fn main() {
    tracing_subscriber::registry().with(fmt::layer()).init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        panic!("Usage: slave proxy_port master_port");
    }

    let db: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    // 创建一个Proxy2Server RPC server
    let proxy_rpc_addr: SocketAddr = (format!("127.0.0.1:{}", &args[1])).parse().unwrap();
    let addr = volo::net::Address::from(proxy_rpc_addr);
    tokio::task::spawn(ScServiceServer::new(Proxy2SlaveService::new(db.clone())).run(addr));

    debug!("rpc server for proxy running at:{}", &args[1]);

    // 创建一个Master2Slave RPC server
    let master_rpc_addr: SocketAddr = (format!("127.0.0.1:{}", &args[2])).parse().unwrap();
    let addr = volo::net::Address::from(master_rpc_addr);
    tokio::task::spawn(Master2SlaveServer::new(Master2SlaveService::new(db.clone())).run(addr));
    
    debug!("rpc server for master running at:{}", &args[2]);
    
    // 异步读取配置文件，获取master的rpc端口号
    let conf_path = "../../config/ms.conf";
    let mut conf_file = File::open(conf_path).await.unwrap();

    debug!("read configure");

    let mut lines = String::new();
    let _ = conf_file.read_to_string(&mut lines).await;
    // 创建一个Slave2Master RPC Client
    let master_addr = String::from(lines.split_whitespace().next().unwrap());
    let master_addr: SocketAddr = master_addr.parse().unwrap();
    let client = Slave2MasterClientBuilder::new("slave")
        .address(master_addr)
        .build();

    debug!("rpc client connect to {}", master_addr);
    register_at_master(&client, &(format!{"127.0.0.1:{}", &args[2]})).await;

    match signal::ctrl_c().await {
        Ok(()) => {
            logout(&client, &(format!{"127.0.0.1:{}", &args[2]})).await;
        },
        Err(err) => {
            error!("{}", err);
        }
    }
}

async fn register_at_master(client: &Slave2MasterClient, addr: &String) {
    let req = PingRequest {
        payload: Some(FastStr::new(format!("{}", addr))),
    };
    let _ = client.register(req).await.expect("Failed to connecting master");
    debug!("slave register success");
}

async fn logout(client: &Slave2MasterClient, addr: &String) {
    let req = PingRequest {
        payload: Some(FastStr::new(format!("{}", addr))),
    };
    debug!("slave logout success");
    let _ = client.logout(req).await.expect("Failed to connecting master");
}