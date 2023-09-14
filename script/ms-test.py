#!/bin/python3

import subprocess
import os
import time
import sys

# 根据配置文件启动主从server
REDIS_PATH = os.environ["MINIREDIS_PATH"]
CONF_PATH = REDIS_PATH + "/config/ms-test.conf"
EXEC_DIR = REDIS_PATH + "/target/debug"
f = open(CONF_PATH)
lines = f.readlines()
master = True
for line in lines:
    line = line.strip()
    bin = EXEC_DIR + "/" + line + " |& grep -e DEBUG | grep -v VOLO" 
    args = ['bash', '-c', bin]
    subprocess.Popen(args)
    if master:
        time.sleep(1)
        master = False

time.sleep(1)

# 启动测试client
bin = "cargo test -q --bin client-test ms_test"
subprocess.Popen(bin.split())