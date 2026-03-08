# Multi-stage Dockerfile for ClipShare Server
# Stage 1: Build the server binary
FROM rust:1.75-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /build

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./
COPY clip_server/Cargo.toml ./clip_server/
COPY clip_server/src ./clip_server/src

# Build the server binary in release mode
RUN cargo build --release --bin clip_server

# Stage 2: Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1000 clipshare

# Set working directory
WORKDIR /app

# Copy the binary from builder
COPY --from=builder /build/target/release/clip_server /app/clip_server

# Change ownership to clipshare user
RUN chown -R clipshare:clipshare /app

# Switch to non-root user
USER clipshare

# Expose the server port
EXPOSE 3000

# Set environment variables
ENV CLIPSHARE_TOKEN=""
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/clipboard || exit 1

# Run the server
CMD ["sh", "-c", "if [ -z \"$CLIPSHARE_TOKEN\" ]; then echo \"Error: CLIPSHARE_TOKEN environment variable is required\"; exit 1; fi && /app/clip_server"]
