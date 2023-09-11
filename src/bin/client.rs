use anyhow::Ok;
use lazy_static::lazy_static;
use pilota::FastStr;
use std::net::SocketAddr;

use volo_gen::volo::example::{
    PingRequest,
    SetRequest,
    GetRequest,
    DelRequest,
};

lazy_static! {
    static ref CLIENT: volo_gen::volo::example::ItemServiceClient = {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        volo_gen::volo::example::ItemServiceClientBuilder::new("volo-example")
            .address(addr)
            .build()
    };
}

#[volo::main]
async fn main() {
    tracing_subscriber::fmt::init();
}

#[allow(dead_code)]
async fn ping(payload: Option<String>) -> Result<String, anyhow::Error> {
    let req = match payload {
        Some(payload) => PingRequest { payload: Some(FastStr::new(payload)) },
        None => PingRequest { payload: None },
    };
    let res = CLIENT.ping(req).await?;
    Ok(res.payload.into_string())
}

#[allow(dead_code)]
async fn set(key: String, value: String) -> Result<(), anyhow::Error> {
    let req = SetRequest {
        key: FastStr::new(key),
        value: FastStr::new(value),
    };
    let res = CLIENT.set(req).await?;
    println!("{}", res.status.into_string());
    Ok(())
}

#[allow(dead_code)]
async fn get(key: String) -> Result<Option<String>, anyhow::Error> {
    let req = GetRequest {
        key: FastStr::new(key),
    };
    let res = CLIENT.get(req).await?;
    match res.value {
        Some(value) => Ok(Some(value.into_string())),
        None => Ok(None),
    }
}

#[allow(dead_code)]
async fn del(keys: Vec<String>) -> Result<i64, anyhow::Error> {
    let req = DelRequest {
        keys: keys.into_iter().map(|k| FastStr::new(k)).collect(),
    };
    let res = CLIENT.del(req).await?;
    Ok(res.num)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ping_test() {
        assert_eq!(ping(Some(String::from("abc"))).await.unwrap(), "abc");
        assert_eq!(ping(None).await.unwrap(), "PONG");
        assert_eq!(ping(Some(String::from("   hello\nworld   "))).await.unwrap(), "   hello\nworld   ");
    }

    #[tokio::test]
    async fn get_set_del_test() {
        set(String::from("abc"), String::from("def")).await.unwrap();
        set(String::from("hello"), String::from("world")).await.unwrap();
        
        assert_eq!(get(String::from("abc")).await.unwrap(), Some(String::from("def")));
        assert_eq!(get(String::from("hello")).await.unwrap(), Some(String::from("world")));
        assert_eq!(get(String::from("abd")).await.unwrap(), None);

        set(String::from("abc"), String::from("hij")).await.unwrap();
        set(String::from("aaa"), String::from("bbb")).await.unwrap();
        assert_eq!(get(String::from("abc")).await.unwrap(), Some(String::from("hij")));
        
        assert_eq!(del(vec![String::from("abc"), String::from("aaa")]).await.unwrap(), 2);
        assert_eq!(get(String::from("abc")).await.unwrap(), None);
        assert_eq!(get(String::from("aaa")).await.unwrap(), None);

        assert_eq!(del(vec![String::from("hello"), String::from("world")]).await.unwrap(), 1);
        assert_eq!(get(String::from("hello")).await.unwrap(), None);

        assert_eq!(del(vec![]).await.unwrap(), 0);
    }
}