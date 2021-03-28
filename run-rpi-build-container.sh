#!/usr/bin/env bash
set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

readonly IMG_TAG=rpi-build-container
readonly CONTAINER_NAME=rpi-build

docker rm ${CONTAINER_NAME} -f
docker build -t ${IMG_TAG} .
docker run -d -t --name ${CONTAINER_NAME} -v "$PWD:/usr/src/app" ${IMG_TAG}
docker ps -a