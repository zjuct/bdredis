use pilota::FastStr;

#[allow(unused_imports)]
use volo_gen::rds::{
    PingRequest,
    SetRequest,
    GetRequest,
    DelRequest,
    ScServiceClient,
    SetTransRequest, GetTransRequest,
	TransResponse,
	MultiResponse, ExecResponse,
};

#[allow(unused_imports)]
use tokio::io::AsyncReadExt;

#[volo::main]
async fn main() {
    tracing_subscriber::fmt::init();
}


#[allow(dead_code)]
async fn ping(client: &ScServiceClient, payload: Option<String>) -> Result<String, anyhow::Error> {
    let req = match payload {
        Some(payload) => PingRequest { payload: Some(FastStr::new(payload)) },
        None => PingRequest { payload: None },
    };
    let res = client.ping(req).await?;
    Ok(res.payload.into_string())
}

#[allow(dead_code)]
async fn set(client: &ScServiceClient, key: String, value: String) -> Result<(), anyhow::Error> {
    let req = SetRequest {
        key: FastStr::new(key),
        value: FastStr::new(value),
    };
    let res = client.set(req).await?;
    println!("{}", res.status.into_string());
    Ok(())
}

#[allow(dead_code)]
async fn get(client: &ScServiceClient, key: String) -> Result<Option<String>, anyhow::Error> {
    let req = GetRequest {
        key: FastStr::new(key),
    };
    let res = client.get(req).await?;
    match res.value {
        Some(value) => Ok(Some(value.into_string())),
        None => Ok(None),
    }
}

#[allow(dead_code)]
async fn del(client: &ScServiceClient, keys: Vec<String>) -> Result<i64, anyhow::Error> {
    let req = DelRequest {
        keys: keys.into_iter().map(|k| FastStr::new(k)).collect(),
    };
    let res = client.del(req).await?;
    Ok(res.num)
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    #[tokio::test]
    async fn aof_test_stage1() {
        let addr: std::net::SocketAddr = "127.0.0.1:18000".parse().unwrap();
        let client = volo_gen::rds::ScServiceClientBuilder::new("client")
            .address(addr)
            .build();

        set(&client, String::from("abc"), String::from("def")).await.unwrap();
        set(&client, String::from("hello"), String::from("world")).await.unwrap();
        set(&client, String::from("a"), String::from("123")).await.unwrap();
        set(&client, String::from("abc"), String::from("aaa")).await.unwrap();
        del(&client, vec![String::from("hello")]).await.unwrap();
    }

    #[tokio::test]
    async fn aof_test_stage2() {
        let redis_path = std::env::var("MINIREDIS_PATH").expect("Please set the MINIREDIS_PATH environment vairable");

        let addr: std::net::SocketAddr = "127.0.0.1:18000".parse().unwrap();
        let client = volo_gen::rds::ScServiceClientBuilder::new("client")
            .address(addr)
            .build();

        assert_eq!(get(&client, String::from("abc")).await.unwrap(), Some(String::from("aaa")));
        assert_eq!(get(&client, String::from("a")).await.unwrap(), Some(String::from("123")));
        assert_eq!(get(&client, String::from("hello")).await.unwrap(), None);

        let mut halt = Command::new(format!("{}/script/halt.py", redis_path));
        let _ = std::process::Command::status(&mut halt).unwrap();
    }

    #[tokio::test]
    async fn ms_test() {
        let redis_path = std::env::var("MINIREDIS_PATH").expect("Please set the MINIREDIS_PATH environment vairable");
        let conf_path = format!("{}/config/proxy-ms-test.conf", redis_path);
        let mut conf_file = tokio::fs::File::open(conf_path).await.unwrap();

        let mut lines = String::new();
        let _ = conf_file.read_to_string(&mut lines).await;
        let mut lines_split = lines.split_whitespace();
        let master_addr: std::net::SocketAddr = String::from(lines_split.next().unwrap()).parse().unwrap();
        let slaves: Vec<&str> = lines_split.collect();

        let mclient = volo_gen::rds::ScServiceClientBuilder::new("mclient")
            .address(master_addr)
            .build();

        let sclients: Vec<ScServiceClient> = slaves.into_iter()
            .map(|s| -> std::net::SocketAddr {
                s.parse().unwrap()
            })
            .map(|a| {
                volo_gen::rds::ScServiceClientBuilder::new("sclient")
                    .address(a)
                    .build()
            })
            .collect();

        // 对master进行set和del
        set(&mclient, String::from("hello"), String::from("world")).await.unwrap();
        set(&mclient, String::from("aaa"), String::from("bbb")).await.unwrap();
        set(&mclient, String::from("123"), String::from("456")).await.unwrap();
        set(&mclient, String::from("abc"), String::from("def")).await.unwrap();
        del(&mclient, vec![String::from("abc")]).await.unwrap();


        // 对slave进行set和del，应当返回error
        for sc in sclients.iter() {
            let e = set(&sc, String::from("a"), String::from("b")).await.expect_err("Should be error");
            println!("{e}");
            let e = del(&sc, vec![String::from("hello")]).await.expect_err("Should be error");
            println!("{e}");
        }

        // 对master进行get
        assert_eq!(get(&mclient, String::from("hello")).await.unwrap(), Some(String::from("world")));
        assert_eq!(get(&mclient, String::from("aaa")).await.unwrap(), Some(String::from("bbb")));
        assert_eq!(get(&mclient, String::from("123")).await.unwrap(), Some(String::from("456")));
        assert_eq!(get(&mclient, String::from("abc")).await.unwrap(), None);

        // 对slave进行get，应当能返回master中set的内容
        for sc in sclients.iter() {
            assert_eq!(get(sc, String::from("hello")).await.unwrap(), Some(String::from("world")));
            assert_eq!(get(sc, String::from("aaa")).await.unwrap(), Some(String::from("bbb")));
            assert_eq!(get(sc, String::from("123")).await.unwrap(), Some(String::from("456")));
            assert_eq!(get(sc, String::from("abc")).await.unwrap(), None);
        }

        let mut halt = Command::new(format!("{}/script/halt.py", redis_path));
        let _ = std::process::Command::status(&mut halt).unwrap();
    }

    #[tokio::test]
    async fn proxy_test() {
        let addr: std::net::SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let client = volo_gen::rds::ScServiceClientBuilder::new("client")
            .address(addr)
            .build();

        // 发送1000条set
        for i in 0..1000 {
            set(&client, format!("key{i}"), format!("value{i}")).await.unwrap();
        }

        // 发送1000条get
        for i in 0..1000 {
            assert_eq!(get(&client, format!("key{i}")).await.unwrap(), Some(format!("value{i}")));
        }
    }

    #[tokio::test]
    async fn transaction_test() {
        let addr: std::net::SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let client = volo_gen::rds::ScServiceClientBuilder::new("client")
            .address(addr)
            .build();

        let trans_id = client.multi(GetTransRequest { key: FastStr::from("begin"), id: -1 }).await.unwrap();
        let trans_id = trans_id.id;
        client.set_trans(SetTransRequest { key: FastStr::from("hello"), value: FastStr::from("world"), id: trans_id }).await.unwrap();
        client.set_trans(SetTransRequest { key: FastStr::from("hello2"), value: FastStr::from("world2"), id: trans_id }).await.unwrap();
        client.set_trans(SetTransRequest { key: FastStr::from("hello3"), value: FastStr::from("world3"), id: trans_id }).await.unwrap();
        client.get_trans(GetTransRequest { key: FastStr::from("hello"), id: trans_id }).await.unwrap();
        client.get_trans(GetTransRequest { key: FastStr::from("hello2"), id: trans_id }).await.unwrap();
        client.get_trans(GetTransRequest { key: FastStr::from("hello3"), id: trans_id }).await.unwrap();
        let resp = client.exec(GetTransRequest { key: FastStr::from("end"), id: trans_id }).await.unwrap();
        assert_eq!(&resp.values[0], "world");
        assert_eq!(&resp.values[1], "world2");
        assert_eq!(&resp.values[2], "world3");
    }
}