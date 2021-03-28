#!/bin/zsh

watch -n 1 "curl -s -X GET ${1}:8080/procs | jq"
