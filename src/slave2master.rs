// Slave2Master
// Master作为RPC Client, Slave作为RPC Server
use anyhow::{anyhow, Ok};
use pilota::FastStr;
use tracing_subscriber::fmt::format;
use volo_gen::rds::{
    Slave2Master,
    PingRequest, PingResponse,
    Master2SlaveClient, Master2SlaveClientBuilder,
};
use volo_thrift::AnyhowError;

use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;

use tracing::debug;

pub struct Slave2MasterService {
    db: Arc<Mutex<HashMap<String, String>>>,
    slaves: Arc<Mutex<HashMap<String, Master2SlaveClient>>>,
}

impl Slave2MasterService {
    pub fn new(slaves: Arc<Mutex<HashMap<String, Master2SlaveClient>>>, db: Arc<Mutex<HashMap<String, String>>>) -> Self {
        Self { db, slaves }
    }
}

#[volo::async_trait]
impl Slave2Master for Slave2MasterService{
	async fn register(&self, _req: PingRequest) ->
        Result<PingResponse, AnyhowError>{
        match _req.payload {
            Some(payload) => {
                let mut t = self.slaves.lock().await;
                let addr: SocketAddr = payload.parse().unwrap();
                let client = Master2SlaveClientBuilder::new("master")
                    .address(addr)
                    .build();

                let _ = (*t).insert(payload.clone().into_string(), client);
                debug!("slave {} registered", payload.clone().into_string());

                // 向slave同步已有的数据
                let db = self.db.lock().await;
                let mut aof = String::new();
                for (key, value) in db.iter() {
                    aof.push_str(&format!("set {} {}\n", key, value));
                }
                println!("{aof}");
                let req = PingRequest {
                    payload: Some(FastStr::from(aof))
                };
                let _ = (*t).get(&payload.into_string()).unwrap().aofsync(req).await.unwrap();
                println!("aofsync success");

                Ok(PingResponse { payload: FastStr::new("register success") })
            },
            None => {
                Err(anyhow!("Empty register request"))
            }
        }
    }
	async fn logout(&self, _req: PingRequest) ->
        Result<PingResponse, AnyhowError>{
        match _req.payload {
            Some(payload) => {
                let mut t = self.slaves.lock().await;

                let _ = (*t).remove(&payload.clone().into_string());
                debug!("slave {} logout", payload.into_string());
                Ok(PingResponse { payload: FastStr::new("logout success") })
            },
            None => {
                Err(anyhow!("Empty logout request"))
            }
        }
    }
}
