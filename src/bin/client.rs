use lazy_static::lazy_static;
use pilota::FastStr;
use std::net::SocketAddr;
use anyhow::anyhow;
use tokio::sync::broadcast;
use tokio::sync::Mutex;

use volo_gen::rds::{
    PingRequest,
    SetRequest,
    GetRequest,
    DelRequest,
    GetTransRequest,
    SetTransRequest,
};

lazy_static! {
    static ref CLIENT: volo_gen::rds::ScServiceClient = {
        // 此处8080应改为proxy上rpc server端口
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        volo_gen::rds::ScServiceClientBuilder::new("client")
            .address(addr)
            .build()
    };

    static ref INTX: Mutex<bool> = Mutex::new(false);
    static ref TXID: Mutex<i64> = Mutex::new(-1);
}

#[derive(Clone, Debug)]
enum Input {
    Ping(Option<String>),
    Set(String, String),
    Get(String),
    Del(Vec<String>),
    Multi,
    Exec,
    Watch(String),
}

#[volo::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // spawn a thread to handle user input
    let (send, mut recv): (broadcast::Sender<Input>, broadcast::Receiver<Input>) = broadcast::channel(16); 
    tokio::task::spawn(get_input(send));

    loop {
        match recv.recv().await {
            Ok(input) => handle_input(input).await,
            Err(_) => break,
        }
    }
}

async fn get_input(send: broadcast::Sender<Input>) {
    let mut quit = false;
    while !quit {
        let mut buf = String::new();
        std::io::stdin()
            .read_line(&mut buf)
            .unwrap();

        
        let input_vec: Vec<&str> = buf.split_whitespace().collect();
        if input_vec.len() == 0 {
            continue;
        }
        match input_vec[0].to_uppercase().as_str() {
            "PING" => {
                match input_vec.len() {
                    1 => {
                        send.send(Input::Ping(None)).unwrap();
                    },
                    2 => {
                        send.send(Input::Ping(Some(String::from(input_vec[1])))).unwrap();
                    },
                    _ => {
                        println!("Invalid format of PING");
                    }
                }
            },
            "SET" => {
                match input_vec.len() {
                    3 => {
                        send.send(Input::Set(String::from(input_vec[1]), String::from(input_vec[2]))).unwrap();
                    },
                    _ => {
                        println!("Invalid format of SET");
                    }
                }
            },
            "GET" => {
                match input_vec.len() {
                    2 => {
                        send.send(Input::Get(String::from(input_vec[1]))).unwrap();
                    },
                    _ => {
                        println!("Invalid format of GET");
                    }
                }
            },
            "DEL" => {
                match input_vec.len() {
                    1 => {
                        println!("Invalid format of DEL");
                    },
                    _ => {
                        send.send(Input::Del(
                            input_vec[1..].iter().map(|s| String::from(*s)).collect()
                        )).unwrap();
                    }
                }
            },
            "MULTI" => {
                match input_vec.len() {
                    1 => {
                        if *INTX.lock().await == true {
                            println!("Already inside transaction");
                        } else {
                            send.send(Input::Multi).unwrap();
                        }
                    },
                    _ => {
                        println!("Invalid format of MULTI");
                    }
                }
            },
            "EXEC" => {
                match input_vec.len() {
                    1 => {
                        if *INTX.lock().await == false {
                            println!("Exec can only be used inside transaction");
                        } else {
                            send.send(Input::Exec).unwrap();
                        }
                    },
                    _ => {
                        println!("Invalid format of EXEC");
                    }
                }
            },
            "WATCH" => {
                match input_vec.len() {
                    2 => {
                        if *INTX.lock().await == false {
                            println!("Watch can only be used inside transaction");
                        } else {
                            send.send(Input::Watch(String::from(input_vec[1]))).unwrap();
                        }
                    },
                    _ => {
                        println!("Invalid format of WATCH");
                    }
                }
            }
            "QUIT" => {
                quit = true;
            },
            _ => {
                println!("Invalid input");
            }
        }
    }
}

async fn handle_input(input: Input) {
    match input {
        Input::Ping(payload) => {
            let res = ping(payload).await.unwrap();
            println!("{}", res);
        },
        Input::Set(key, value) => {
            set(key, value).await.unwrap();
        },
        Input::Get(key) => {
            let res = get(key).await.unwrap();
            if !*INTX.lock().await {
                match res {
                    Some(value) => {
                        println!("{value}");
                    },
                    None => {
                        println!("{{nil}}");
                    }
                }
            }
        },
        Input::Del(key) => {
            let res = del(key).await.unwrap();
            println!("{res}");
        },
        Input::Multi => {
            multi().await.unwrap();
            println!("Transaction start");
        },
        Input::Exec => {
            match exec().await {
                Ok(values) => {
                    println!("Transaction end");
                    for value in values {
                        println!("{value}");
                    }
                },
                Err(_) => {
                    println!("Watch violated");
                    println!("Transaction interrupted");
                }
            }
        },
        Input::Watch(key) => {
            watch(key).await.unwrap();
        }
    }
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
    if *INTX.lock().await {
        // inside transaction
        let req = SetTransRequest {
            key: FastStr::new(key),
            value: FastStr::new(value),
            id: *TXID.lock().await,
        };
        let _ = CLIENT.set_trans(req).await?;
        Ok(())
    } else {
        let req = SetRequest {
            key: FastStr::new(key),
            value: FastStr::new(value),
        };
        let res = CLIENT.set(req).await?;
        println!("{}", res.status.into_string());
        Ok(())
    }
}

#[allow(dead_code)]
async fn get(key: String) -> Result<Option<String>, anyhow::Error> {
    if *INTX.lock().await {
        // inside transaction
        let req = GetTransRequest {
            key: FastStr::new(key),
            id: *TXID.lock().await,
        };

        let _ = CLIENT.get_trans(req).await?;
        Ok(Some(String::from("Ok")))
    } else {
        let req = GetRequest {
            key: FastStr::new(key),
        };
        let res = CLIENT.get(req).await?;
        match res.value {
            Some(value) => Ok(Some(value.into_string())),
            None => Ok(None),
        }
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

#[allow(dead_code)]
async fn multi() -> Result<(), anyhow::Error> {
    assert!(!*INTX.lock().await);
    let req = GetTransRequest {
        key: FastStr::from("Start tx"),
        id: -1,
    };
    let res = CLIENT.multi(req).await?;
    *TXID.lock().await = res.id;
    *INTX.lock().await = true;
    Ok(())
}

#[allow(dead_code)]
async fn exec() -> Result<Vec<String>, anyhow::Error> {
    assert!(*INTX.lock().await);
    let req = GetTransRequest {
        key: FastStr::from("End tx"),
        id: *TXID.lock().await,
    };
    let res = CLIENT.exec(req).await;
    *TXID.lock().await = -1;
    *INTX.lock().await = false;
    match res {
        Ok(values) => {
            Ok(values.values.into_iter().map(|s| s.into_string()).collect())
        },
        Err(_) => {
            Err(anyhow!("watch vialated"))
        }
    }
}

#[allow(dead_code)]
async fn watch(key: String) -> Result<(), anyhow::Error> {
    assert!(*INTX.lock().await);
    let req = GetTransRequest {
        key: FastStr::from(key),
        id: *TXID.lock().await,
    };
    let _ = CLIENT.watch(req).await?;
    Ok(())
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