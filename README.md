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
docker build -t gql-analyzer:latest .
docker run --rm -it -d -p 3002:3002 gql-analyzer:latest
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
docker image build -t router-proxy:latest .
docker run --rm -it -p 3004:3004 router-proxy:latest
./router --config router.yaml --supergraph supergraph-schema.graphql
```

```sh
kind load docker-image gql-analyzer:latest
kind load docker-image router-proxy:latest
```

```sh
kubectl port-forward gql-analysis-68b755b7b9-drwhk 3004:3004
kubectl exec -it gql-analysis-68b755b7b9-drwhk -c router-main sh

kubectl logs gql-analysis-68b755b7b9-drwhk -c router-main
kubectl logs gql-analysis-68b755b7b9-drwhk -c gql-analyzer-sidecar
```