FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
ENV CARGO_HOME=/cargo
ENV RUSTUP_HOME=/rustup
ENV PATH=/cargo/bin:$PATH

RUN apt-get update -yq && apt-get install -yq --no-install-recommends \
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
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install rustup + Rust 1.96 (MSRV declared in rust-toolchain.toml)
ARG RUST_VERSION=1.96
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --no-modify-path --profile minimal --default-toolchain none && \
    rustup toolchain install ${RUST_VERSION} \
        --profile minimal \
        --component rustfmt \
        --component clippy && \
    rustup default ${RUST_VERSION}

WORKDIR /workspace

VOLUME ["/workspace", "/cargo/registry"]
