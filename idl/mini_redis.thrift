namespace rs rds

// PING PONG
struct PingRequest {
    1: optional string payload,
}

struct PingResponse {
    1: required string payload,
}

// SET
struct SetRequest {
    1: required string key,
    2: required string value,
}

struct SetResponse {
    1: required string status,
}

// GET
struct GetRequest {
    1: required string key,
}

struct GetResponse {
    1: optional string value,
}

// DEL
struct DelRequest {
    1: required list<string> keys,
}

struct DelResponse {
    1: required i64 num,
}

// Client-Proxy && Proxy-Server
service SCService {
    PingResponse ping(1: PingRequest req),
    SetResponse set(1: SetRequest req),
    GetResponse get(1: GetRequest req),
    DelResponse del(1: DelRequest req),
}

// Master-Slave
service Slave2Master {
    PingResponse register(1: PingRequest req)
    PingResponse logout(1: PingRequest req)
}

service Master2Slave {
    PingResponse aofsync(1: PingRequest req)
    PingResponse rdbsync(1: PingRequest req)
}