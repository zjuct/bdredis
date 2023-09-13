#![feature(impl_trait_in_assoc_type)]

use std::net::SocketAddr;

use bdredis::{
    proxy2server::Proxy2ServerService,
    slave2master::Slave2MasterService,
    master2slave::Master2SlaveService,
};
use volo_gen::rds::{
    ScServiceServer,
    Slave2MasterServer, 
    Master2SlaveClient, Master2SlaveClientBuilder,
};

#[volo::main]
async fn main() {
    let addr: SocketAddr = "[::]:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    ScServiceServer::new(Proxy2ServerService { })
        .run(addr)
        .await
        .unwrap();

    let addr: SocketAddr = "[::]:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);
    Slave2MasterServer::new(Slave2MasterService {})
        .run(addr)
        .await
        .unwrap();

    // 根据master/slave配置文件创建多个Master2Slave RPC Client
}
