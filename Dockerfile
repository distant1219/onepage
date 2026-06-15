# Build stage
FROM rust:1.86-slim AS builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests (include Cargo.lock for reproducible builds)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies cache layer
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY src ./src
COPY templates ./templates
COPY static ./static

# Remove the placeholder crate artifacts so the real sources actually recompile.
# Without this, cargo's mtime-based fingerprint can decide the copied-in sources
# aren't "newer" than the dummy build and skip recompilation — leaving the empty
# `fn main() {}` binary in the final image. Dependency .rlibs stay cached.
RUN rm -f target/release/onepage target/release/deps/onepage-*

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies.
# wget is required by the HEALTHCHECK below — bookworm-slim does not ship it.
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Copy binary and assets
COPY --from=builder /app/target/release/onepage /app/onepage
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/static /app/static

# Create config directory
RUN mkdir -p /app/config

# Expose port
EXPOSE 8080

# Set environment
ENV RUST_LOG=info
ENV CONFIG_PATH=/app/config/onepage.toml

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

ENTRYPOINT ["/app/onepage"]
