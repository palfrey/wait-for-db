#!/bin/bash

set -eux -o pipefail

if [ ! -f /proc/sys/fs/binfmt_misc/qemu-aarch64 ]; then
    docker run --rm --privileged multiarch/qemu-user-static --reset -p yes
fi

docker rm wait_for_db || true

if [ ! -f wait-for-db-linux-amd64 ]; then
    docker buildx build --platform linux/amd64 --build-arg HOST=x86_64-unknown-linux-musl --tag wait_for_db --output=type=docker .
    docker run --platform linux/amd64 --name wait_for_db wait_for_db --help
    docker cp wait_for_db:/wait_for_db wait-for-db-linux-amd64
    docker stop wait_for_db
    docker rm wait_for_db
fi

if [ ! -f wait-for-db-linux-arm64 ]; then
    docker buildx build --platform linux/arm64 --build-arg HOST=aarch64-unknown-linux-musl --tag wait_for_db --output=type=docker .
    docker run --platform linux/arm64 --name wait_for_db wait_for_db --help
    docker cp wait_for_db:/wait_for_db wait-for-db-linux-arm64
    docker stop wait_for_db
    docker rm wait_for_db
fi