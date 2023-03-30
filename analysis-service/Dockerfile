# Create a new, lightweight base image for the final layer
FROM debian:buster-slim

# Set the working directory
WORKDIR /usr/src/app

# Install the necessary dependencies
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates libssl-dev \
  && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY /target/wasm32-wasi/debug/gql-analysis.wasm /usr/src/app/gql-analysis.wasm
COPY /wasmtime-dev-x86_64-linux/wasmtime /usr/local/bin/wasmtime

# Expose the port your application uses (change it to the port you're using)
EXPOSE 8080

# Start the application
CMD ["wasmtime", "run", "--tcplisten", "0.0.0.0:8080", "--env", "FD_COUNT=3", "gql-analysis.wasm"]

# docker run --rm -it -d -p 3000:8080 elcrazy/gql-analysis:10.0
