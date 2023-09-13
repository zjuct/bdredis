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

pub struct Client2ProxyService;

#[volo::async_trait]
impl ScService for Client2ProxyService {
	async fn ping(&self, _req: PingRequest) ->
		::core::result::Result<PingResponse, ::volo_thrift::AnyhowError> {
        Err(anyhow!("TODO"))
	}

	async fn set(&self, _req: SetRequest) ->
		::core::result::Result<SetResponse, ::volo_thrift::AnyhowError> {
        Err(anyhow!("TODO"))
	}

	async fn get(&self, _req: GetRequest) ->
		::core::result::Result<GetResponse, ::volo_thrift::AnyhowError> {
        Err(anyhow!("TODO"))
	}

	async fn del(&self, _req: DelRequest) ->
		::core::result::Result<DelResponse, ::volo_thrift::AnyhowError> {
        Err(anyhow!("TODO"))
	}
}