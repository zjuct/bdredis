use anyhow::{anyhow, Ok};
// Proxy2Server
// Proxy作为RPC client, Server(Master/Slave)作为RPC server
use volo_gen::rds::{
    ScService,
    PingRequest, PingResponse,
    SetRequest, SetResponse,
    GetRequest, GetResponse,
    DelRequest, DelResponse,
};

use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

pub struct Proxy2SlaveService {
	db: Arc<Mutex<HashMap<String, String>>>,
}

impl Proxy2SlaveService {
	pub fn new(db: Arc<Mutex<HashMap<String, String>>>) -> Self {
		Self { db }
	}
}


#[volo::async_trait]
impl ScService for Proxy2SlaveService {
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
		::core::result::Result<SetResponse, ::volo_thrift::AnyhowError> {
        Err(anyhow!("Can only send SET to master"))
	}

	async fn get(&self, _req: GetRequest) ->
		::core::result::Result<GetResponse, ::volo_thrift::AnyhowError> {
		let t = self.db.lock().await;
		let res = (*t).get(&_req.key.into_string());

		match res {
			Some(value) => Ok(GetResponse { value: Some(value.clone().parse().unwrap()) }),
			None => Ok(GetResponse { value: None })
		}
	}

	async fn del(&self, _req: DelRequest) ->
		::core::result::Result<DelResponse, ::volo_thrift::AnyhowError> {
        Err(anyhow!("Can only send DEL to master"))
	}
}