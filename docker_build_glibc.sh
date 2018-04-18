#!/bin/sh

CONTAINER_NAME="custom/rustnightly:nightly-2018-02-14"

[ ! "$(docker ps -a | grep $CONTAINER_NAME)" ] && docker build -t $CONTAINER_NAME .

docker run -v $PWD:/usr/src/build -t $CONTAINER_NAME bash -c "cd /usr/src/build && cargo build --release"
