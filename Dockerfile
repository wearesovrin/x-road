# syntax=docker/dockerfile:1

# -----------------------------------------------------------------------------
# Base builder stage
# -----------------------------------------------------------------------------
FROM rust:1.85-bookworm AS builder

# Install build dependencies (protobuf-compiler for prost/tonic)
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/x-road

# Copy manifests and source tree
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release binaries for security-server, central-server, and x-road CLI
RUN cargo build --release -p security-server -p central-server -p x-road -p asic

# -----------------------------------------------------------------------------
# Runtime stage for security-server
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS security-server

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    tzdata \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Non-root service user
RUN groupadd -g 10001 xroad && \
    useradd -u 10001 -g xroad -s /bin/bash -m xroad

WORKDIR /etc/x-road
RUN chown -R xroad:xroad /etc/x-road

USER xroad

COPY --from=builder /usr/src/x-road/target/release/security-server /usr/local/bin/security-server
COPY --from=builder /usr/src/x-road/target/release/x-road /usr/local/bin/x-road
COPY --from=builder /usr/src/x-road/target/release/xroad-cli /usr/local/bin/xroad-cli

# Expose Security Server REST Management API (4000), proxies (8000, 8443), and native mTLS (5500)
EXPOSE 4000 8000 8443 5500

ENV RUST_LOG=info \
    ADMIN_BIND_ADDR=0.0.0.0:4000 \
    CLIENT_PROXY_BIND_ADDR=0.0.0.0:8000 \
    SERVER_PROXY_BIND_ADDR=0.0.0.0:8443 \
    MTLS_BIND_ADDR=0.0.0.0:5500

ENTRYPOINT ["/usr/local/bin/security-server"]

# -----------------------------------------------------------------------------
# Runtime stage for central-server
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS central-server

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    tzdata \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Non-root service user
RUN groupadd -g 10001 xroad && \
    useradd -u 10001 -g xroad -s /bin/bash -m xroad

WORKDIR /etc/x-road
RUN chown -R xroad:xroad /etc/x-road

USER xroad

COPY --from=builder /usr/src/x-road/target/release/central-server /usr/local/bin/central-server

# Expose Central Server Management API (4001) and Global Configuration distribution (80)
EXPOSE 4001 80

ENV RUST_LOG=info \
    BIND_ADDR=0.0.0.0:4001 \
    XROAD_INSTANCE=DEV

ENTRYPOINT ["/usr/local/bin/central-server"]
