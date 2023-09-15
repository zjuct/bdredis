# mini-redis

- 支持`PING`, `GET`, `SET`, `DEL`, `MULTI`, `WATCH`, `EXEC`命令

- 提供client-cli命令行前端

- 实现AOF

- 实现master/slave架构

- 实现cluster

## 服务器启动方式

1. 设置环境变量`MINIREDIS_PATH`为bdredis目录地址(重要)
2. `config/ms.conf`为master/slave配置文件，格式如下

```
master port_for_proxy port_for_slave
slave port_for_proxy port_for_master master_port
slave port_for_proxy port_for_master master_port
slave port_for_proxy port_for_master master_port
```
以上启动了1个master和3个slave，注意`port_for_slave`应与`master_port`相等

> `ms.conf`示例
> ```
> master 18000 10000
> slave 18001 10001 10000
> slave 18002 10002 10000
> slave 18003 10003 10000
> ```
    
3. 运行`script/bootstrap.py`，启动所有server和proxy

4. 在另一个窗口中运行`cargo run --bin client`，启动client

5. 可以使用`script/halt.py`关闭所有server和proxy

## client-cli使用

### `PING`

```
PING [string]
```

### `GET`, `SET`, `DEL`

```
GET key
SET key value
DEL [key]...
```


## 测试

- 运行`script/aof-test.py`，进行AOF测试
- 运行`script/ms-test.py`，进行master/slave测试
- 运行`script/proxy-test.py`，进行cluster测试

