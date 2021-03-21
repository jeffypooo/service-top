#!/usr/bin/env bash

# shellcheck disable=SC2034
for i in {0..1000}
do
    curl --request GET \
    --url http://localhost:8080/test
    clear
done