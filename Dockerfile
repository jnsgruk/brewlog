# syntax=docker/dockerfile:1

# Multi-stage Dockerfile for brewlog.
# Uses chisel to create a minimal Ubuntu rootfs for the runtime image.

# ---------------------------------------------------------------------------
# Builder — Ubuntu with Rust toolchain pre-installed
# ---------------------------------------------------------------------------
FROM ubuntu/rust:1.93-26.04_edge AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    mold \
    binutils \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install tailwindcss standalone (needed by build.rs)
RUN mkdir -p /usr/local/bin \
    && curl -sL https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64 \
    -o /usr/local/bin/tailwindcss && chmod +x /usr/local/bin/tailwindcss

WORKDIR /app
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked \
    && mkdir -p /out \
    && cp target/release/brewlog /out/brewlog

# ---------------------------------------------------------------------------
# Chisel — minimal Ubuntu rootfs
# ---------------------------------------------------------------------------
FROM ubuntu:26.04 AS chisel
ARG TARGETARCH=amd64
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN curl -sL "https://github.com/canonical/chisel/releases/download/v1.4.1/chisel_v1.4.1_linux_${TARGETARCH}.tar.gz" \
    | tar xz -C /usr/local/bin

RUN mkdir /rootfs && chisel cut --root /rootfs \
    base-files_base \
    base-files_release-info \
    ca-certificates_data \
    libgcc-s1_libs \
    libc6_libs \
    libssl3t64_libs \
    openssl_bins

# ---------------------------------------------------------------------------
# Runtime — scratch with chisel rootfs
# ---------------------------------------------------------------------------
FROM scratch
COPY --from=chisel /rootfs /
COPY --from=builder /out/brewlog /usr/local/bin/brewlog
USER 65534:65534
ENTRYPOINT ["brewlog", "serve", "--database-url", "sqlite:///data/brewlog.db"]
