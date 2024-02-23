#!/bin/bash

rm -rf docs

cargo clean

cargo doc --document-private-items --no-deps

cp -r target/doc/* ../docs

cargo clean

cd ..
