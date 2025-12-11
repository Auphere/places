# ============================================
# STAGE 1: Build Stage - Compile Rust binary
# ============================================
FROM rust:latest AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy all source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN cargo build --release --locked

# ============================================
# STAGE 2: Runtime Stage - Minimal image
# ============================================
FROM debian:bookworm-slim

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd -m -u 1001 -s /bin/bash auphere

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/auphere-places /usr/local/bin/auphere-places
COPY --from=builder /app/migrations ./migrations

# Change ownership
RUN chown -R auphere:auphere /app

# Switch to non-root user
USER auphere

# Expose port 8002
EXPOSE 8002

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8002/health || exit 1

# Run the binary
CMD ["auphere-places"]
