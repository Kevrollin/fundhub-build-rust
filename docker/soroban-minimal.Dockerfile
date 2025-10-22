# Minimal Soroban CLI Docker image using pre-built binary
FROM alpine:latest

# Install dependencies
RUN apk add --no-cache \
    curl \
    ca-certificates \
    bash

# Download pre-built Soroban CLI binary (if available) or use a simple approach
RUN curl -L https://github.com/stellar/soroban-tools/releases/latest/download/soroban-cli-linux-x86_64.tar.gz -o soroban.tar.gz \
    && tar -xzf soroban.tar.gz \
    && mv soroban /usr/local/bin/ \
    && chmod +x /usr/local/bin/soroban \
    && rm soroban.tar.gz

# Set working directory
WORKDIR /workspace

# Default command
CMD ["soroban", "--help"]
