use pilota::FastStr;
// Proxy2Server
// Proxy作为RPC client, Server(Master/Slave)作为RPC server
use volo_gen::rds::{
    ScService,
    PingRequest, PingResponse,
    SetRequest, SetResponse,
    GetRequest, GetResponse,
    DelRequest, DelResponse,
	Master2SlaveClient,
};

use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

use tracing::debug;

pub struct Proxy2MasterService {
	db: Arc<Mutex<HashMap<String, String>>>,
	slaves: Arc<Mutex<HashMap<String, Master2SlaveClient>>>,
}

impl Proxy2MasterService {
	pub fn new(db: Arc<Mutex<HashMap<String, String>>>, slaves: Arc<Mutex<HashMap<String, Master2SlaveClient>>>) -> Self {
		Self { db, slaves }
	}
}

#[volo::async_trait]
impl ScService for Proxy2MasterService {
	async fn ping(&self, _req: PingRequest) ->
		::core::result::Result<PingResponse, ::volo_thrift::AnyhowError> {
		debug!("PING {:?}", _req);
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
		debug!("SET {:?}", _req);
		let mut t = self.db.lock().await;
		let _ = (*t).insert(_req.key.clone().into_string(), _req.value.clone().into_string());

		let req = format!("set {} {}", _req.key.into_string(), _req.value.into_string());
		let req = PingRequest {
			payload: Some(FastStr::new(req)),
		};

		debug!("aofsync start");
		for (_, client) in self.slaves.lock().await.iter() {
			// 这里用await不太好
			client.aofsync(req.clone()).await.unwrap();
		}

		Ok(SetResponse { status: "OK".parse().unwrap() })
	}

	async fn get(&self, _req: GetRequest) ->
		::core::result::Result<GetResponse, ::volo_thrift::AnyhowError> {
		debug!("GET {:?}", _req);
		let t = self.db.lock().await;
		let res = (*t).get(&_req.key.into_string());

		match res {
			Some(value) => Ok(GetResponse { value: Some(value.clone().parse().unwrap()) }),
			None => Ok(GetResponse { value: None })
		}
	}

	async fn del(&self, _req: DelRequest) ->
		::core::result::Result<DelResponse, ::volo_thrift::AnyhowError> {
		debug!("DEL {:?}", _req);
		let mut t = self.db.lock().await;
		let mut num = 0;
		
		for key in &_req.keys {
			if let Some(_) = (*t).remove(key.as_str()) {
				num += 1;
			}
		}

		let mut req = String::from("del");
		for key in _req.keys {
			req.push_str(" ");
			req.push_str(&key.into_string());
		}
		let req = PingRequest {
			payload: Some(FastStr::new(req)),
		};
		debug!("aofsync start");
		for (_, client) in self.slaves.lock().await.iter() {
			// 这里用await不太好
			client.aofsync(req.clone()).await.unwrap();
		}

		Ok(DelResponse { num })
	}
}