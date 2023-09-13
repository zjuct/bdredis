// Slave2Master
// Master作为RPC Client, Slave作为RPC Server
use anyhow::anyhow;
use volo_gen::rds::{
    Slave2Master,
    PingRequest, PingResponse,
};
use volo_thrift::AnyhowError;

pub struct Slave2MasterService;


#[volo::async_trait]
impl Slave2Master for Slave2MasterService{
	async fn register(&self, _req: PingRequest) ->
        Result<PingResponse, AnyhowError>{
        Err(anyhow!("TODO"))
    }
	async fn logout(&self, _req: PingRequest) ->
        Result<PingResponse, AnyhowError>{
        Err(anyhow!("TODO"))
    }
}
