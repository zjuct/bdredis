use anyhow::{anyhow, Ok};
// Proxy2Server
// Proxy作为RPC client, Server(Master/Slave)作为RPC server
use volo_gen::rds::{
    ScService,
    PingRequest, PingResponse,
    SetRequest, SetResponse,
    GetRequest, GetResponse,
    DelRequest, DelResponse,
	SetTransRequest, GetTransRequest,
	TransResponse,
	MultiResponse, ExecResponse,
};

use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

use tracing::debug;

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
		debug!("PING");
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
		debug!("SET");
        Err(anyhow!("Can only send SET to master"))
	}

	async fn get(&self, _req: GetRequest) ->
		::core::result::Result<GetResponse, ::volo_thrift::AnyhowError> {
		debug!("GET");
		let t = self.db.lock().await;
		let res = (*t).get(&_req.key.into_string());

		match res {
			Some(value) => Ok(GetResponse { value: Some(value.clone().parse().unwrap()) }),
			None => Ok(GetResponse { value: None })
		}
	}

	async fn del(&self, _req: DelRequest) ->
		::core::result::Result<DelResponse, ::volo_thrift::AnyhowError> {
		debug!("DEL");
        Err(anyhow!("Can only send DEL to master"))
	}

	async fn set_trans(&self, _req: SetTransRequest) ->
		::core::result::Result<TransResponse, ::volo_thrift::AnyhowError> {
        Err(anyhow!("NOT IMPLEMENTED"))
	}

	async fn get_trans(&self, _req: GetTransRequest) ->
		::core::result::Result<TransResponse, ::volo_thrift::AnyhowError> {
        Err(anyhow!("NOT IMPLEMENTED"))
	}

	async fn multi(&self, _req: GetTransRequest) ->
		::core::result::Result<MultiResponse, ::volo_thrift::AnyhowError> {
        Err(anyhow!("NOT IMPLEMENTED"))
	}

	async fn exec(&self, _req: GetTransRequest) ->
		::core::result::Result<ExecResponse, ::volo_thrift::AnyhowError> {
        Err(anyhow!("NOT IMPLEMENTED"))
	}

	async fn watch(&self, _req: GetTransRequest) ->
		::core::result::Result<TransResponse, ::volo_thrift::AnyhowError> {
        Err(anyhow!("NOT IMPLEMENTED"))
	}
}