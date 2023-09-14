
// Client2Proxy 
// Client作为RPC client, Proxy作为RPC server
use anyhow::anyhow;
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
use std::net::SocketAddr;//, thread};
//use crate::proxy2server::Proxy2ServerService;
use std::hash::{Hash,Hasher};

use std::collections::hash_map::DefaultHasher;

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;

use pilota::FastStr;

use tracing::debug;
//use crate::proxy2server;

pub struct Client2ProxyService{
	master: volo_gen::rds::ScServiceClient,
	slaves: Vec<volo_gen::rds::ScServiceClient>,
	hash_trans: Arc<Mutex<HashMap<i64, (Vec<String>,bool)>>>, //tuple 
	hash_watch: Arc<Mutex<HashMap<String,i64>>>,
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
			hash_trans: Arc::new(Mutex::new(HashMap::new())),
			hash_watch: Arc::new(Mutex::new(HashMap::new())),
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
		//first decide if it has conflicted
		let  db_watch = self.hash_watch.lock().await;
		if let Some(trans_id) = db_watch.get(&_req.key.to_string()){
			let mut db_trans = self.hash_trans.lock().await; //to change the state
			if let Some((trans,state)) = db_trans.get_mut(&trans_id){
				*state = false;
			}else{  //insert an empty value with state false
				let trans = Vec::new();

				db_trans.insert(*trans_id, (trans,false));
			}
		}
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
			Ok(resp) =>{
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

	async fn set_trans(&self, _req: SetTransRequest) ->
		::core::result::Result<TransResponse, ::volo_thrift::AnyhowError> {
		
		debug!("SET_TRANS: {:?}", _req);
        let trans_id = _req.id;
		let command = format!("{} {}",_req.key,_req.value);
		let mut db_trans = self.hash_trans.lock().await;
		if let Some((trans,state)) = db_trans.get_mut(&trans_id){
			if *state{
				trans.push(command);
			}
		}else{
			let mut trans = Vec::new();
			trans.push(command);
			db_trans.insert(trans_id, (trans,true));
		}
		Ok(TransResponse { status: FastStr::from("set") })
	}

	async fn get_trans(&self, _req: GetTransRequest) ->
		::core::result::Result<TransResponse, ::volo_thrift::AnyhowError> {
			debug!("GET_TRANS: {:?}", _req);
			let trans_id = _req.id;
			let command = format!("{} get",_req.key);
			let mut db_trans = self.hash_trans.lock().await;
			if let Some((trans,state)) = db_trans.get_mut(&trans_id){
				if *state{
					trans.push(command);
				}
			}else{
				let mut trans = Vec::new();
				trans.push(command);
				db_trans.insert(trans_id, (trans,true));
			}
			Ok(TransResponse { status: FastStr::from("get") })
	}

	async fn multi(&self, _req: GetTransRequest) ->
		::core::result::Result<MultiResponse, ::volo_thrift::AnyhowError> {
		debug!("MULTI: {:?}", _req);
		let db_trans = self.hash_trans.lock().await;
		let id = (db_trans.keys().len()) as i64; //new id
		Ok(MultiResponse{
			id
		})
	}

	async fn exec(&self, _req: GetTransRequest) ->
		::core::result::Result<ExecResponse, ::volo_thrift::AnyhowError> {
		debug!("EXEC: {:?}", _req);
		let trans_id = _req.id;
		let mut db_trans = self.hash_trans.lock().await;

		let mut result_vec = Vec::new();
		if let Some((trans,state)) = db_trans.get_mut(&trans_id){
			if *state{  //not conflicted
				for comm in trans{
					let command = comm.clone();
					let com_split:Vec<String> = command.split_whitespace().map(|s| String::from(s)).collect();
					//let com_split = com_split.clone();
					if com_split[1] == "get"{
						let resp = self.get(GetRequest { key: FastStr::from(com_split[0].clone()) }).await?;
						match resp.value {
							Some(value) => {
								result_vec.push(value);
							},
							None => {
	
							},
						}
					}else{  //set
						let _ = self.set(SetRequest { key: FastStr::from(com_split[0].clone()), value: FastStr::from(com_split[1].clone()) }).await;
					}
				}
			} else {
				//remove the key-value
				db_trans.remove(&trans_id);
				
				//remove db_watch too
				let mut db_watch = self.hash_watch.lock().await;

				let keys_to_remove: Vec<_> = db_watch
				.iter()
				.filter(|&(_, value)| *value == trans_id)
				.map(|(key, _)| key.clone())
				.collect();

				for key in keys_to_remove {
					db_watch.remove(&key);
				}
				return Err(anyhow!("Watch vialated"));
			}

			//remove the key-value
			db_trans.remove(&trans_id);
			
			//remove db_watch too
			let mut db_watch = self.hash_watch.lock().await;

			let keys_to_remove: Vec<_> = db_watch
        	.iter()
        	.filter(|&(_, value)| *value == trans_id)
        	.map(|(key, _)| key.clone())
        	.collect();

    		for key in keys_to_remove {
    		    db_watch.remove(&key);
    		}

		}else{
			return Err(anyhow!("NO Result"))
			// let mut trans = Vec::new();
			// trans.push(command);
			// db_trans.insert(trans_id, trans);
		}
		Ok(ExecResponse { values: result_vec })
		
	}

	async fn watch(&self, _req: GetTransRequest) ->
		::core::result::Result<TransResponse, ::volo_thrift::AnyhowError> {
		let mut db_watch = self.hash_watch.lock().await;
		let trans_id = _req.id;
		db_watch.insert(_req.key.to_string(), trans_id); //register
		Ok(TransResponse { status: FastStr::from("Success") })
	}
}