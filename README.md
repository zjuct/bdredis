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

3. `config/proxy.conf`为proxy配置文件，格式如下

```
127.0.0.1:18000
127.0.0.1:18001
127.0.0.1:18002
127.0.0.1:18003
```

以上对应1个master和3个salve，注意`config/proxy.conf`中的端口号与`config/ms.conf`中的`port_for_proxy`端口相同
    
4. 运行`script/bootstrap.py`，启动所有server和proxy

5. 在另一个窗口中运行`cargo run --bin client`，启动client

6. 可以使用`script/halt.py`关闭所有server和proxy

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

