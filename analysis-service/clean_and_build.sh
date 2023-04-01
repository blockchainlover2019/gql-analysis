#!/bin/bash

echo "Cleaning..."
cargo clean
echo "Updating..."
cargo update
echo "Building..."
cargo build --target=wasm32-wasi
