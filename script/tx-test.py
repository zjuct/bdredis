#!/bin/python3

import os
import subprocess
import time

# 启动master-slave
REDIS_PATH = os.environ["MINIREDIS_PATH"]
CONF_PATH = REDIS_PATH + "/config/ms.conf"
EXEC_DIR = REDIS_PATH + "/target/debug"
f = open(CONF_PATH)
master = True
lines = f.readlines()
for idx, line in enumerate(lines):
    line = line.strip()
    bin = f'{EXEC_DIR}/{line}'
    subprocess.Popen(bin.split())

    if master:
        time.sleep(1)
        master = False

time.sleep(1)

# 启动proxy
proxy = EXEC_DIR + "/proxy"
args = ['bash', '-c', proxy]
f = open("/dev/null")
subprocess.Popen(args)

time.sleep(1)

# 启动client
bin = "cargo test -q --bin client-test transaction_test"
subprocess.Popen(bin.split())
