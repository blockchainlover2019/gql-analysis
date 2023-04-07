FROM wasmedge/slim:0.11.2

# Set the working directory
WORKDIR /usr/src/app

# Copy the compiled binary from the builder stage
COPY /target/wasm32-wasi/release/gql-analysis.wasm /usr/src/app/gql-analysis.wasm
COPY sec_config.yaml /usr/src/app/sec_config.yaml

# Expose the port your application uses (change it to the port you're using)
EXPOSE 3005

# Start the application
CMD ["wasmedge", "--dir=.", "gql-analysis.wasm"]
