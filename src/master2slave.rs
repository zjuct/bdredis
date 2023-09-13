// Master2Slave
// Slave作为RPC Client, Master作为RPC Server
use anyhow::anyhow;
use volo_gen::rds::{
    Master2Slave,
    PingRequest, PingResponse,
};
use volo_thrift::AnyhowError;

pub struct Master2SlaveService;


#[volo::async_trait]
impl Master2Slave for Master2SlaveService {
	async fn aofsync(&self, _req: PingRequest) ->
        Result<PingResponse, AnyhowError>{
        Err(anyhow!("TODO"))
    }
	async fn rdbsync(&self, _req: PingRequest) ->
        Result<PingResponse, AnyhowError>{
        Err(anyhow!("TODO"))
    }
}
