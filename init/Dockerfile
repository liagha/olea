FROM rust:slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    musl-tools \
    pkg-config

# Add Rust target for static linking
RUN rustup target add x86_64-unknown-linux-musl

# Create a new empty project
RUN USER=root cargo new --bin init
WORKDIR /init

# Copy the source code and Cargo.toml
COPY ./src/init.rs ./src/main.rs
COPY ./Cargo.toml ./Cargo.toml

# Build for release with static linking
RUN cargo build --release --target x86_64-unknown-linux-musl

# Final stage
FROM scratch
COPY --from=builder /init/target/x86_64-unknown-linux-musl/release/init /init