#!/usr/bin/env bash

HOST='localhost'
#HOST='192.168.1.69'
PORT='8080'
URL="http://${HOST}:${PORT}/test"

# shellcheck disable=SC2034
for i in {0..1000}
do
#    curl -s -o /dev/null http://localhost:8080/test -w "%{http_code} in %{time_total}\n"
    curl -s -o /dev/null http://192.168.1.69:8080/test -w "%{http_code} in %{time_total}\n"
done
