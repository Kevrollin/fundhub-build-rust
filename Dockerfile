# Build stage
FROM rust:1.85-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    file \
    autoconf \
    automake \
    libtool \
    make \
    gcc \
    g++ \
    libsodium-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies first
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached unless Cargo.toml changes)
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsodium23 \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from builder
COPY --from=builder /app/target/release/fundhub /app/fundhub
COPY --from=builder /app/migrations /app/migrations

# Expose port
EXPOSE 8000

# Run the application
CMD ["/app/fundhub"]

