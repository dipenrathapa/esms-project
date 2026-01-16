# ═══════════════════════════════════════════════════════════════════════════════
# Environmental Stress Monitoring System (ESMS) - Dockerfile
# ═══════════════════════════════════════════════════════════════════════════════
# Multi-stage build for optimized production image

# ───────────────────────────────────────────────────────────────────────────────
# Stage 1: Build
# ───────────────────────────────────────────────────────────────────────────────
# FROM rust:1.75-alpine AS builder
# FROM rust:1.78-alpine AS builder
FROM rust:1.86-alpine AS builder



# Install build dependencies
RUN apk add --no-cache musl-dev pkgconfig openssl-dev

# Create app directory
WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs for dependency compilation
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Build dependencies only (cached layer)
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY src ./src

# Touch main.rs to invalidate the build cache for our code
RUN touch src/main.rs

# Build the application
RUN cargo build --release --bin esms

# ───────────────────────────────────────────────────────────────────────────────
# Stage 2: Runtime
# ───────────────────────────────────────────────────────────────────────────────
FROM alpine:3.19 AS runtime

# Install runtime dependencies
RUN apk add --no-cache ca-certificates tzdata

# Create non-root user for security
RUN addgroup -g 1000 esms && \
    adduser -u 1000 -G esms -h /app -D esms

WORKDIR /app

# Copy built binary from builder
COPY --from=builder /app/target/release/esms /app/esms

# Copy configuration
COPY .env.example /app/.env.example

# Set ownership
RUN chown -R esms:esms /app

# Switch to non-root user
USER esms

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/api/health || exit 1

# Run the application
CMD ["./esms"]
