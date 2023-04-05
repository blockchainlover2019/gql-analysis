# Use the official Rust image as a base image
FROM rust:1.68.2 as builder

# Set the working directory
WORKDIR /usr/src/app

# Copy your source code and the Cargo.toml file
COPY src src
COPY Cargo.toml Cargo.toml

# Build the release version of the application
RUN cargo build --release

# Create a new, lightweight base image for the final layer
FROM debian:buster-slim

# Install the necessary dependencies
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/gql-service /usr/local/bin/gql-service

# Expose the port your application uses (change it to the port you're using)
EXPOSE 27017

# Start the application
CMD ["gql-service"]