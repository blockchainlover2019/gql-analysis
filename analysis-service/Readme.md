## build

### For standalone test
```sh
cargo build --target=wasm32-wasi --release --features=standalone
```

### For docker test
```sh
cargo build --target=wasm32-wasi --release
```

### Copy config files and run
```
cp sec_config.yaml target/wasm32-wasi/release/sec_config.yaml
wasmedge --dir=. target/wasm32-wasi/release/gql-analysis.wasm
```