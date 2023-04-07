cargo build --target=wasm32-wasi --release
#cargo rustc --release --target wasm32-wasi -- -Z wasi-exec-model=command
cp config.yaml target/wasm32-wasi/release/config.yaml
wasmedge --dir=. target/wasm32-wasi/release/gql-analysis.wasm
