# Use a rust base image
FROM rust:1.70 as builder

# Create a new empty shell project
RUN USER=root cargo new --bin bugcrowd-webhook-manager
WORKDIR /bugcrowd-webhook-manager

# Copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Build your dependencies
RUN cargo build --release
RUN rm src/*.rs

# Copy your source tree
COPY ./src ./src

# Build for release.
# Time-stamp and debug info is stripped for smaller binary
RUN rm ./target/release/deps/bugcrowd_webhook_manager*
RUN cargo build --release

# Our final base
FROM debian:buster-slim

# Install OpenSSL
RUN apt-get update \
    && apt-get install -y ca-certificates tzdata libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the build artifact from the builder stage
COPY --from=builder /bugcrowd-webhook-manager/target/release/bugcrowd-webhook-manager .

EXPOSE 3000

ENV PORT 3000

# Set the binary as the entrypoint of the container
ENTRYPOINT ["./bugcrowd-webhook-manager"]
