#![feature(impl_trait_in_assoc_type)]

use std::net::SocketAddr;
use tracing::debug;

use bdredis::{
    client2proxy::Client2ProxyService,
    //proxy2master::Proxy2MasterService,
    //proxy2slave::Proxy2SlaveService,
};
    
use volo_gen::rds::{
    ScServiceServer,
    //ScServiceClient, ScServiceClientBuilder,
};

use tracing_subscriber::{fmt, util::SubscriberInitExt, prelude::__tracing_subscriber_SubscriberExt};

use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[volo::main]
async fn main() {
    tracing_subscriber::registry().with(fmt::layer()).init();
    let conf_path = "../../config/proxy.conf";
    let mut conf_file = File::open(conf_path).await.unwrap();

    debug!("read configure");

    let mut lines = String::new();
    let _ = conf_file.read_to_string(&mut lines).await;
    let mut lines_split = lines.split_whitespace();
    let master_addr = String::from(lines_split.next().unwrap());
    //let master_addr: SocketAddr = master_addr.parse().unwrap();
    let slaves:Vec<&str> = lines_split.collect();
    let proxy = Client2ProxyService::new(&master_addr,&slaves);
    
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);
    debug!("proxy begin");
    // TODO: 不能用await
    ScServiceServer::new(proxy)
        .run(addr)
        .await
        .unwrap();

    
    // 读取proxy配置文件，起多个RPC Client
}
