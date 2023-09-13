#![feature(impl_trait_in_assoc_type)]

use std::net::SocketAddr;

use bdredis::{
    proxy2server::Proxy2ServerService,
    slave2master::Slave2MasterService,
    master2slave::Master2SlaveService,
};
use volo_gen::rds::{
    ScServiceServer,
    Slave2MasterClient, Slave2MasterClientBuilder,
    Master2SlaveServer,
};


#[volo::main]
async fn main() {
    let addr: SocketAddr = "[::]:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    // TODO: 创建一个Proxy2Server RPC server
    // TODO: 创建一个Master2Slave RPC server
    // TODO: 创建一个Slave2Master RPC Client
}
