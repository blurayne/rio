# AGENTS.md — Rio Build Environment

## Overview

Rio is a GPU-accelerated terminal emulator written in Rust. The canonical build
environment is an **Ubuntu 22.04 container** initialized via Docker Compose.
All builds, tests, and linting must run inside this container — never on the
host directly.

The Nix flake (`flake.nix` / `pkgRio.nix`) is the authoritative source for
required native dependencies; the `Dockerfile` translates those into Ubuntu
`apt` packages.

---

## Prerequisites

- Docker ≥ 24
- Docker Compose v2 (`docker compose`)
- (Optional) Nix + direnv for host-side tooling / IDE support

---

## Quick Start

```bash
# Build the image (first time or after Dockerfile changes)
docker compose build

# Drop into an interactive shell inside the container
docker compose run --rm build

# Run a one-shot command without entering a shell
docker compose run --rm build cargo build --release -p rioterm --no-default-features --features=wayland
```

---

## Container Details

| Item | Value |
|------|-------|
| Base image | `ubuntu:22.04` |
| Rust toolchain | pinned via `rust-toolchain.toml` (currently `1.96`, `minimal` profile + `rustfmt` + `clippy`) |
| Cargo home | `/cargo` (persisted as named Docker volume) |
| Source mount | `.` → `/workspace` |
| Build cache | `target/` persisted as named Docker volume |

### System dependencies (derived from `pkgRio.nix` + CI apt installs)

| Package | Purpose |
|---------|---------|
| `build-essential`, `cmake`, `pkg-config` | C/C++ toolchain, build system |
| `libfontconfig1-dev` | Font enumeration |
| `libgl1-mesa-dev`, `libgles2-mesa-dev` | OpenGL (software fallback) |
| `glslang-tools` | GLSL shader compilation (shaderc equivalent) |
| `libvulkan-dev` | Vulkan loader |
| `libxkbcommon-dev`, `libxkbcommon-x11-dev` | Keyboard handling |
| `libx11-dev`, `libxcursor-dev`, `libxi-dev`, `libxrandr-dev`, `libxcb1-dev` | X11 support |
| `libwayland-dev` | Wayland support |
| `libasound2-dev` | Audio (ALSA) |

---

## Build Commands

All commands run inside the container (`docker compose run --rm build <cmd>`).

### Release build (Wayland — default Linux target)

```bash
cargo build --release -p rioterm --no-default-features --features=wayland
```

### Release build (X11)

```bash
cargo build --release -p rioterm --no-default-features --features=x11
```

### Debug build

```bash
cargo build -p rioterm --no-default-features --features=wayland
```

### Debian package

```bash
# requires cargo-deb
cargo install cargo-deb
cargo deb -p rioterm -- --no-default-features --features=wayland
```

---

## Test & Lint Commands

```bash
# Format check
cargo fmt -- --check --color always

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Tests (includes wgpu feature gate, mirrors CI)
cargo test --features wgpu
```

Full suite (mirrors `make test`):

```bash
docker compose run --rm build bash -c "
  cargo fmt -- --check --color always &&
  cargo clippy --all-targets --all-features -- -D warnings &&
  RUST_BACKTRACE=full cargo test --features wgpu
"
```

---

## Nix Environment (host-side, optional)

`flake.nix` provides a dev shell with the exact same native deps. Useful for
IDE integration or if you prefer Nix over Docker:

```bash
# Requires nix with flakes + direnv
direnv allow        # auto-activates via .envrc → `use flake`

# Or manually
nix develop
```

The flake exposes multiple toolchain variants:

| Shell | Toolchain |
|-------|-----------|
| `nix develop` | MSRV (same as `rust-toolchain.toml`) |
| `nix develop .#stable` | Latest stable |
| `nix develop .#nightly` | Latest nightly |

---

## Volume Layout

```
cargo-registry   →  /cargo/registry   (crate source cache, shared across runs)
cargo-git        →  /cargo/git        (git-sourced crates)
target-cache     →  /workspace/target (incremental build artifacts)
```

Wipe build cache without losing crate downloads:

```bash
docker volume rm rio_target-cache
```

---

## Feature Flags

| Flag | Description |
|------|-------------|
| `wayland` | Wayland compositor support (default on Linux) |
| `x11` | X11/XCB support |
| `wgpu` | WebGPU renderer (required for `cargo test`) |

Flags are mutually exclusive for production builds; both can be compiled
simultaneously for testing (`--all-features`).
