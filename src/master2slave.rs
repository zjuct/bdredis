// Master2Slave
// Slave作为RPC Client, Master作为RPC Server
use anyhow::anyhow;
use pilota::FastStr;
use volo_gen::rds::{
    Master2Slave,
    PingRequest, PingResponse,
};
use volo_thrift::AnyhowError;

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;

use tracing::debug;

pub struct Master2SlaveService {
    db: Arc<Mutex<HashMap<String, String>>>,
}

impl Master2SlaveService {
    pub fn new(db: Arc<Mutex<HashMap<String, String>>>) -> Self {
        Self { db }
    }
}


#[volo::async_trait]
impl Master2Slave for Master2SlaveService {
	async fn aofsync(&self, _req: PingRequest) ->
        Result<PingResponse, AnyhowError>{
        debug!("aofsync {:?}", _req);
        let aofs: Vec<String> = _req.payload.expect("Empty aof request").into_string()
            .split("\n")
            .map(|s| String::from(s))
            .collect();


        let mut t = self.db.lock().await;

        for aof in aofs {
            let mut iter = aof.split(" ");
            match iter.next().unwrap() {
                "set" => {
                    let key = String::from(iter.next().unwrap());
                    let value = String::from(iter.next().unwrap());
                    let _ = (*t).insert(key, value);
                },
                "del" => {
                    for key in iter {
                        let _ = (*t).remove(key);
                    }
                },
                _ => {
                    return Err(anyhow!("Unrecognized AOF request"));
                }
            }
        }
        Ok(PingResponse { payload: FastStr::new("aofsync finish") })
    }

	async fn rdbsync(&self, _req: PingRequest) ->
        Result<PingResponse, AnyhowError>{
        debug!("rdbsync");
        Err(anyhow!("TODO"))
    }
}
