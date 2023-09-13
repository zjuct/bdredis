#![feature(impl_trait_in_assoc_type)]

use std::net::SocketAddr;
use std::env;

use bdredis::{
    proxy2server::Proxy2ServerService,
    slave2master::Slave2MasterService,
    master2slave::Master2SlaveService,
};
use tracing::debug;
use volo::net::dial::Config;
use volo_gen::rds::{
    ScServiceServer,
    Slave2MasterClient, Slave2MasterClientBuilder,
    Master2SlaveServer,
};

use tracing_subscriber::{fmt, util::SubscriberInitExt, prelude::__tracing_subscriber_SubscriberExt};


#[volo::main]
async fn main() {
    tracing_subscriber::registry().with(fmt::layer()).init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        panic!("Usage: slave proxy_port master_port");
    }

    // TODO: 创建一个Proxy2Server RPC server
    let proxy_rpc_port: SocketAddr = (&args[1]).parse().unwrap();
    let addr = volo::net::Address::from(proxy_rpc_port);
    let proxy_rpc_server_hdl = ScServiceServer::new(Proxy2ServerService { })
        .run(addr);

    debug!("rpc server for proxy running at:{}", &args[1]);

    // TODO: 创建一个Master2Slave RPC server
    let master_rpc_port: SocketAddr = (&args[2]).parse().unwrap();
    
    
    
    let addr: SocketAddr = "[::]:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    // TODO: 创建一个Slave2Master RPC Client
}
