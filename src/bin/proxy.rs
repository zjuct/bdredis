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
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: proxy conf")
    }

    let redis_path = std::env::var("MINIREDIS_PATH").expect("Please set the MINIREDIS_PATH environment vairable");
    let conf_path = format!("{}/config/{}", redis_path, &args[1]);
    let mut conf_file = File::open(conf_path).await.unwrap();

    debug!("read configure");

    let mut lines = String::new();
    let _ = conf_file.read_to_string(&mut lines).await;
    let mut lines_split = lines.split_whitespace();
    let master_line = lines_split.next().unwrap();
    let master_vec:Vec<&str> = master_line.split_whitespace().collect();

    let master_addr = format!("127.0.0.1:{}",master_vec[1]);

    let slave_lines:Vec<&str> = lines_split.collect();
    let mut slaves = Vec::new();
    for line in slave_lines{
        let comm:Vec<&str> = line.split_whitespace().collect();
        slaves.push(format!("127.0.0.1:{}",comm[1]));
    }
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
