#!/bin/sh

docker run -v $PWD:/volume -t clux/muslrust:1.25.0-nightly-2018-02-14 cargo build --release
# docker run -v $PWD:/volume -t clux/muslrust:1.25.0-nightly-2018-02-14 nightly cargo build --release
