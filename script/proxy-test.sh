#!/bin/bash

for i in {1..10}
do
    cat proxy-test/server$i.log | grep DEBUG | grep get | wc -l
done