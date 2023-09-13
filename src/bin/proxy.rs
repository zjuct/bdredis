#![feature(impl_trait_in_assoc_type)]

use std::net::SocketAddr;

use bdredis::{
    client2proxy::Client2ProxyService,
    proxy2master::Proxy2MasterService,
    proxy2slave::Proxy2SlaveService,
};
    
use volo_gen::rds::{
    ScServiceServer,
    ScServiceClient, ScServiceClientBuilder,
};

#[volo::main]
async fn main() {
    let addr: SocketAddr = "[::]:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    // TODO: 不能用await
    ScServiceServer::new(Client2ProxyService {})
        .run(addr)
        .await
        .unwrap();

    
    // 读取proxy配置文件，起多个RPC Client
}
