
// Client2Proxy 
// Client作为RPC client, Proxy作为RPC server
use anyhow::anyhow;
use volo_gen::rds::{
    ScService,
    PingRequest, PingResponse,
    SetRequest, SetResponse,
    GetRequest, GetResponse,
    DelRequest, DelResponse,
};
use std::net::SocketAddr;//, thread};
//use crate::proxy2server::Proxy2ServerService;
use std::hash::{Hash,Hasher};

use std::collections::hash_map::DefaultHasher;

//use crate::proxy2server;

pub struct Client2ProxyService{
	master: volo_gen::rds::ScServiceClient,
	slaves: Vec<volo_gen::rds::ScServiceClient>,
}
impl Client2ProxyService {
	pub fn new(master_addr:&str, slaves_addr: &Vec<&str>)->Self{
		let addr: SocketAddr = master_addr.parse().unwrap();
        let mas = volo_gen::rds::ScServiceClientBuilder::new("master")
            .address(addr).build();
		let mut sla_v:Vec<volo_gen::rds::ScServiceClient> = Vec::new();
		for slave in slaves_addr{
			let addr: SocketAddr = slave.parse().unwrap();
			let mut name = String::from("slave");
			name.push_str(slave);
        	let sla = volo_gen::rds::ScServiceClientBuilder::new(name)
            .address(addr).build();
			sla_v.push(sla);
		}
		Client2ProxyService{
			master: mas,
			slaves: sla_v,
		}
	}
	fn my_hash(&self, input:&str)->usize{
		let mut hasher = DefaultHasher::new();
		input.hash(&mut hasher);
		hasher.finish() as usize % self.slaves.len()
	}
}
#[volo::async_trait]
impl ScService for Client2ProxyService {
	async fn ping(&self, _req: PingRequest) ->
		::core::result::Result<PingResponse, ::volo_thrift::AnyhowError> {

			match _req.payload {
				Some(payload) => {
					Ok(PingResponse { payload })
				},
				None => {
					Ok(PingResponse { payload: "PONG".parse().unwrap() })
				}
			}
	}

	async fn set(&self, _req: SetRequest) ->
		::core::result::Result<SetResponse, ::volo_thrift::AnyhowError> {  //all to the master
        match self.master.set(_req).await{
			Ok(resp) => {
				Ok(resp)
			}
			Err(_) =>{
				Err(anyhow!("failed"))
			}
		}
	}

	async fn get(&self, _req: GetRequest) ->
		::core::result::Result<GetResponse, ::volo_thrift::AnyhowError> {
        let distubutor = self.my_hash(&_req.key);
		match self.slaves[distubutor].get(_req).await{
			Ok(resp) => {
				Ok(resp)
			}
			Err(_) =>{
				Err(anyhow!("failed"))
			}
		}
	}

	async fn del(&self, _req: DelRequest) ->
		::core::result::Result<DelResponse, ::volo_thrift::AnyhowError> {
			match self.master.del(_req).await{
				Ok(resp) => {
					Ok(resp)
				}
				Err(_) =>{
					Err(anyhow!("failed"))
				}
			}
	}
}