# ========================================================
# LoL Classic Meta - Cross-Platform Build Container
# Tauri v2 + Rust + Node.js 20 + NSIS Support
# Developed for vamp9
# ========================================================

FROM rust:1.82-bookworm AS builder

# Set non-interactive debian frontend
ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo

# 1. Install system prerequisites for Tauri v2 and Windows/Linux builds
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    wget \
    file \
    libwebkit2gtk-4.1-dev \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    nsis \
    mingw-w64 \
    pkg-config \
    ca-certificates \
    gnupg \
    && rm -rf /var/lib/apt/lists/*

# 2. Install Node.js 20 LTS and npm
RUN mkdir -p /etc/apt/keyrings && \
    curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg && \
    echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_20.x nodistro main" | tee /etc/apt/sources.list.d/nodesource.list && \
    apt-get update && apt-get install -y nodejs && \
    rm -rf /var/lib/apt/lists/*

# 3. Add Rust targets for cross-compilation if targeting Windows
RUN rustup target add x86_64-pc-windows-gnu || true

# 4. Set working directory
WORKDIR /app

# 5. Copy package files first to leverage Docker layer caching
COPY package.json package-lock.json* ./
RUN npm install

# 6. Copy source code
COPY . .

# 7. Default entrypoint: build frontend and compile Tauri bundle
CMD ["sh", "-c", "npm run build && npm run tauri build"]
