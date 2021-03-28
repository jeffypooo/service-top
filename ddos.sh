#!/usr/bin/env bash

HOST='localhost'
#HOST='192.168.1.69'
PORT='8080'
URL="http://${HOST}:${PORT}/test"

# shellcheck disable=SC2034
for i in {0..1000}
do
    curl -s -o /dev/null --location "${URL}" -w "%{http_code} in %{time_total}\n"
done
