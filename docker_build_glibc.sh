#!/bin/sh

docker run -v $PWD:/usr/src/build -t rustlang/rust:nightly bash -c "cd /usr/src/build && cargo build --release"
