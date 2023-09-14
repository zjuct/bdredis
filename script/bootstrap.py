#!/bin/python3

import os
import subprocess

CONF_PATH = "../config/ms.conf"
EXEC_DIR = "../target/debug"
f = open(CONF_PATH)
lines = f.readlines()
for line in lines:
    line = line.strip()
    bin = EXEC_DIR + "/" + line + " |& grep -e DEBUG" 
    args = ['bash', '-c', bin]
    subprocess.Popen(args)

proxy = EXEC_DIR + "/proxy"
args = ['bash', '-c', proxy]
subprocess.Popen(args)