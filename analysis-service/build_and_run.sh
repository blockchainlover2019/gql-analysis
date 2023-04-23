cargo build --target=wasm32-wasi --release
#cargo rustc --release --target wasm32-wasi -- -Z wasi-exec-model=command
cp target/wasm32-wasi/release/gql-analysis.wasm ../../../Dotnet/graphql/Demo/gql-analysis.wasm
cp sec_config.yaml target/wasm32-wasi/release/sec_config.yaml
wasmedge --dir=. target/wasm32-wasi/release/gql-analysis.wasm
