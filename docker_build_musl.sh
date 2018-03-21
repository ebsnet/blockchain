#!/bin/sh

docker run -v $PWD:/volume -t clux/muslrust:nightly cargo build --release
