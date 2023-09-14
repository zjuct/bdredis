#!/bin/python3

import subprocess
import os

try: 
    print("kill slaves")
    cmd = "ps -C slave -o pid h"
    slave_pids = subprocess.check_output(cmd.split())
    slave_pids = slave_pids.splitlines()

    pids = []
    for pid in slave_pids:
        pids.append(str(pid.strip())[2:-1])

    cmd = "kill -s SIGINT "
    for pid in pids:
        c = cmd + str(pid)
        print(c)
        subprocess.run(c.split())
except:
    print("no slave")


try:
    print("kill masters")
    cmd = "ps -C master -o pid h"
    master_pids = subprocess.check_output(cmd.split())
    master_pids = master_pids.splitlines()

    pids = []
    for pid in master_pids:
        pids.append(str(pid.strip())[2:-1])

    cmd = "kill -s SIGINT "
    for pid in pids:
        c = cmd + str(pid)
        print(c)
        subprocess.run(c.split())
except:
    print("no master")

try:
    print("kill proxy")
    cmd = "ps -C proxy -o pid h"
    proxy_pids = subprocess.check_output(cmd.split())
    proxy_pids = proxy_pids.splitlines()

    pids = []
    for pid in proxy_pids:
        pids.append(str(pid.strip())[2:-1])

    cmd = "kill -s SIGINT "
    for pid in pids:
        c = cmd + str(pid)
        print(c)
        subprocess.run(c.split())
except:
    print("no proxy")