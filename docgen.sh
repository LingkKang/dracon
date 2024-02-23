#!/bin/bash

rm -rf ./docs/*

cargo clean

cargo doc --document-private-items --no-deps

echo '<meta http-equiv="refresh" content="0; url=raft/index.html">' > ./target/doc/index.html

cp -r target/doc/* ./docs

cargo clean
