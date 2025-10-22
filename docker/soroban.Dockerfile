# Ultra-lightweight Soroban CLI Docker image
FROM ubuntu:22.04

# Install minimal dependencies
RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Download and install Rust
RUN curl --proto '=https' --tlsv1.2 -sSfL https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Soroban CLI
RUN cargo install --locked soroban-cli

# Set working directory
WORKDIR /workspace

# Default command
CMD ["soroban", "--help"]
