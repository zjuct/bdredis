use core::marker::{Send};

use tokio::{fs::{self, File},
            sync::Mutex, io::AsyncWriteExt};
use regex::Regex;
type  Buffer = Mutex<String>;

pub struct AOFManager {
    fstream: Mutex<File>,
    file_name: String,
    buffer: Buffer,
}

impl AOFManager 
{   
    pub async fn new(file_name: &'static str) -> Result<Self, anyhow::Error> {
        let buf = Mutex::new(String::new());
        let fname = String::from(file_name);
        let mut fs = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("output.txt")
            .await
            .unwrap();
        Ok(Self {
            fstream: Mutex::new(fs),
            file_name: fname,
            buffer: buf,
        })
    }

    pub async fn flush(&self) -> Result<(), anyhow::Error>{
        let path = self.file_name.as_str();
        let mut buf = self.buffer.lock().await;
        let content = buf.as_bytes();
        let mut fs = self.fstream.lock().await;
        fs.write_all(content).await;
        buf.clear();
        drop(buf);
        Ok(())
    }

    async fn parse(req_str: String) -> Result<String, ()> {

        let re = Regex::new(r"(DelRequest|SetRequest|GetRequest) \{[^}]*\}").unwrap();
        let req = &(re.captures(req_str.as_str()).unwrap()[0]);
        let mut log = String::new();
        if req.starts_with("Del") {
            log.push_str("del");
            let re = Regex::new(r"keys:\s*\[([^\]]+)\]").unwrap();
            let keys_raw = &re.captures(req).unwrap()[1];
            let keys = &keys_raw[1..keys_raw.len()-1];
            let keys: Vec<&str> = keys.split("\", \"").collect();
            for key in keys {
                log.push_str(" ");
                log.push_str(key);
            }
            Ok(log)
        } else if req.starts_with("Set") {
            log.push_str("set");
            let re = Regex::new(r"key: ([^,]+), value: ([^\ ]+)").unwrap();
            if let Some(res) = re.captures(req) {
                let key = &res[1];
                let key = &key[1..key.len()-1];
                log.push_str(" ");
                log.push_str(key);
                let value = &res[2];
                let value = &value[1..value.len()-1];
                log.push_str(" ");
                log.push_str(value);
            }
            Ok(log)
        } else if req.starts_with("Get") {
            log.push_str("get");
            let re = Regex::new(r"key: ([^\ ]+)").unwrap();
            if let Some(res) = re.captures(req) {
                let key = &res[1];
                let key = &key[1..key.len()-1];
                log.push_str(" ");
                log.push_str(key);
            }
            Ok(log)
        } else {
            Err(())
        }
    }

    pub async fn append<Req: std::fmt::Debug + core::marker::Send + 'static>(&self, Request: &Req ) {
        let req = format!("{:?}", Request);
        let log = Self::parse(req).await.unwrap();
        println!("The aof record: {}", log);
        let mut buf = self.buffer.lock().await;
        buf.push_str(log.as_str());
        buf.push_str("\n");
    }
}