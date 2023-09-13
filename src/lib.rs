#![feature(impl_trait_in_assoc_type)]

use pilota::lazy_static::lazy_static;
use volo_gen::volo::example::{
	PingRequest, PingResponse,
	SetRequest, SetResponse,
	GetRequest, GetResponse,	
	DelRequest, DelResponse,
};
use tokio::sync::{
	Mutex,
	broadcast::Sender,
};
use std::collections::HashMap;
use std::sync::Arc;


pub struct S;

lazy_static! {
	static ref DB: Mutex<HashMap<String, String>> = {
		println!("Init DB");
		Mutex::new(HashMap::new())
	};
	static ref CHANNEL: Mutex<HashMap<String, Arc<Sender<String>>>> = {
		println!("Init CHANNLE");
		Mutex::new(HashMap::new())
	};
}

#[volo::async_trait]
impl volo_gen::volo::example::ItemService for S {
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
		let mut t = DB.lock().await;
		let _ = (*t).insert(_req.key.into_string(), _req.value.into_string());
		Ok(SetResponse { status: "OK".parse().unwrap() })
	}

	async fn get(&self, _req: GetRequest) ->
		::core::result::Result<GetResponse, ::volo_thrift::AnyhowError> {
		let t = DB.lock().await;
		let res = (*t).get(&_req.key.into_string());
		
		match res {
			Some(value) => Ok(GetResponse { value: Some(value.clone().parse().unwrap()) }),
			None => Ok(GetResponse { value: None })
		}
	}

	async fn del(&self, _req: DelRequest) ->
		::core::result::Result<DelResponse, ::volo_thrift::AnyhowError> {
		let mut t = DB.lock().await;
		let mut num = 0;
		
		for key in _req.keys {
			if let Some(_) = (*t).remove(key.as_str()) {
				num += 1;
			}
		}
		Ok(DelResponse { num })
	}
}