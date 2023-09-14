#!/bin/python3

import os
import subprocess
import time

REDIS_PATH = os.environ["MINIREDIS_PATH"]
CONF_PATH = REDIS_PATH + "/config/ms.conf"
EXEC_DIR = REDIS_PATH + "/target/debug"
f = open(CONF_PATH)
lines = f.readlines()
master = True
for line in lines:
    line = line.strip()
    bin = EXEC_DIR + "/" + line + " |& grep -e DEBUG" 
    args = ['bash', '-c', bin]
    subprocess.Popen(args)
    
    if master:
        time.sleep(1)
        master = False

time.sleep(1)

proxy = EXEC_DIR + "/proxy"
args = ['bash', '-c', proxy]
subprocess.Popen(args)