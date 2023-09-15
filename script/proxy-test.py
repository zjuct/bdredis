#!/bin/python3

import os
import subprocess
import time


REDIS_PATH = os.environ["MINIREDIS_PATH"]
LOG_DIR = REDIS_PATH + "/script/proxy-test"

bin = "rm -f {REDIS_PATH}/redis.data"
subprocess.Popen(bin.split())
bin = "rm -f {LOG_DIR}/*"
subprocess.Popen(bin.split())
time.sleep(1)

# 启动master-slave
CONF_PATH = REDIS_PATH + "/config/proxy-test.conf"
EXEC_DIR = REDIS_PATH + "/target/debug"
f = open(CONF_PATH)
master = True
lines = f.readlines()
fnull = open("/dev/null")
for idx, line in enumerate(lines):
    flog = open(f'{LOG_DIR}/server{idx}.log', '+w')
    line = line.strip()
    bin = f'{EXEC_DIR}/{line}'
    subprocess.Popen(bin.split(), stderr=fnull, stdout=flog)

    if master:
        time.sleep(2)
        master = False

time.sleep(2)

# 启动proxy
CONF_FILE = "proxy-test.conf"
proxy = EXEC_DIR + "/proxy " + CONF_FILE
args = ['bash', '-c', proxy]
f = open("/dev/null")
subprocess.Popen(args, stderr=fnull, stdout=fnull)

# 启动client
bin = "cargo test -q --bin client-test proxy_test"
f = open("/dev/null")
subprocess.Popen(bin.split(), stderr=fnull, stdout=fnull)

# time.sleep(15)
# 
# # 统计所有slave接受的get数量
# for i in range(1, len(lines)):
#     p1 = subprocess.Popen(["cat", f"{LOG_DIR}/server{i}.log"], stdout=subprocess.PIPE)
#     p2 = subprocess.Popen(["grep", "DEBUG"], stdin=p1.stdout, stdout=subprocess.PIPE)
#     p3 = subprocess.Popen(["grep", "get"], stdin=p2.stdout, stdout=subprocess.PIPE)
#     p4 = subprocess.Popen(["wc", "-l"], stdin=p3.stdout)

