#!/bin/python3

import subprocess
import os
import time
import sys

# 启动server
REDIS_PATH = os.environ["MINIREDIS_PATH"]
# bin = REDIS_PATH + '/target/debug/master 18000 10000 |& grep -e DEBUG | grep -v VOLO' 
bin = REDIS_PATH + '/target/debug/master 18000 10000'
args = ['bash', '-c', bin]
subprocess.Popen(args)

time.sleep(1)

# 往server中注入数据
bin = "cargo test -q --bin client-test aof_test_stage1"
subprocess.Popen(bin.split())

# 等待2s
time.sleep(2)

# 关闭server
bin = REDIS_PATH + '/script/halt.py'
subprocess.Popen(bin.split())

time.sleep(2)

# 重启server
# bin = REDIS_PATH + '/target/debug/master 18000 10000 |& grep -e DEBUG | grep -v VOLO' 
bin = REDIS_PATH + '/target/debug/master 18000 10000'
args = ['bash', '-c', bin]
subprocess.Popen(args)

time.sleep(1)

# 从server中读取数据
bin = "cargo test -q --bin client-test aof_test_stage2"
subprocess.Popen(bin.split())