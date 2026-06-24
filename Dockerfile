# syntax=docker/dockerfile:1.6
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
ENV CARGO_HOME=/cargo
ENV RUSTUP_HOME=/rustup
ENV PATH=/cargo/bin:$PATH

# Keep apt's downloaded .deb files between image rebuilds by:
#  1. Disabling Docker's auto-clean of /var/cache/apt.
#  2. Mounting BuildKit caches over /var/cache/apt + /var/lib/apt so the
#     downloads + package lists persist across builds.
# Result: rebuilding the image after changing a single apt line goes
# from ~minutes to ~seconds (only the delta is re-fetched).
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    rm -f /etc/apt/apt.conf.d/docker-clean && \
    apt-get update -yq && \
    apt-get install -yq --no-install-recommends \
    build-essential \
    cmake \
    curl \
    git \
    pkg-config \
    # fontconfig / text rendering
    libfontconfig1-dev \
    # OpenGL / GPU (software fallback)
    libgl1-mesa-dev \
    libgles2-mesa-dev \
    # Vulkan / shaderc / GLSL
    glslang-tools \
    libvulkan-dev \
    # X11
    libx11-dev \
    libxcursor-dev \
    libxi-dev \
    libxrandr-dev \
    libxcb1-dev \
    libxkbcommon-dev \
    libxkbcommon-x11-dev \
    # Wayland
    libwayland-dev \
    # audio
    libasound2-dev \
    # misc
    ca-certificates

# Install rustup + Rust 1.96 (MSRV declared in rust-toolchain.toml).
# Toolchain itself must live in the image (not in a BuildKit cache)
# so it's available at runtime; rustup re-downloads are infrequent
# (only when pinning to a new toolchain version), so a cache here
# would add complexity without much payoff.
ARG RUST_VERSION=1.96
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --no-modify-path --profile minimal --default-toolchain none && \
    rustup toolchain install ${RUST_VERSION} \
        --profile minimal \
        --component rustfmt \
        --component clippy && \
    rustup default ${RUST_VERSION}

WORKDIR /workspace

# Runtime caches (named volumes wired in docker-compose.yml):
#   /cargo/registry   crate source cache (shared across runs)
#   /cargo/git        git-sourced crates
#   /workspace/target incremental build artifacts
# These are populated on each `docker compose run --rm build cargo …`
# invocation and survive container teardown.
VOLUME ["/workspace", "/cargo/registry"]
