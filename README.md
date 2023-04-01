## run **gql-service**
```sh
cargo run
```

## deploy **analysis-service** as a sidecar of pod
change request ip address to local address
```sh
./build_and_run	
```
```sh
docker build -t gql-analysis:latest .
docker run --rm -it -d -p 3000:3000 gql-analysis:latest
```

deploy to k8s
```sh
kubectl apply -f deployment.yaml
kubectl get pods
kubectl get deployment
kubectl port-forward gql-analysis-7947bc5d9-hclhr 3000:3000
```
## run apollo router
```sh
./router --config router.yaml --supergraph supergraph-schema.graphql
```