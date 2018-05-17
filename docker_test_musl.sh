#!/bin/sh

mkdir -p cargo
docker run -v $PWD:/volume -t clux/muslrust:1.25.0-nightly-2018-02-14 sh -c "CARGO_HOME=./cargo cargo test --release"
